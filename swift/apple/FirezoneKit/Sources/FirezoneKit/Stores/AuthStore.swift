//
//  AuthStore.swift
//  (c) 2024 Firezone, Inc.
//  LICENSE: Apache-2.0
//

import Combine
import Dependencies
import Foundation
import NetworkExtension
import OSLog

#if os(macOS)
  import AppKit
#endif

@MainActor
public final class AuthStore: ObservableObject {
  enum LoginStatus: CustomStringConvertible {
    case uninitialized
    case needsTunnelCreationPermission
    case signedOut
    case signedIn(actorName: String)

    var description: String {
      switch self {
      case .uninitialized:
        return "uninitialized"
      case .needsTunnelCreationPermission:
        return "needsTunnelCreationPermission"
      case .signedOut:
        return "signedOut"
      case .signedIn(let actorName):
        return "signedIn(actorName: \(actorName))"
      }
    }
  }

  @Dependency(\.keychain) private var keychain
  @Dependency(\.auth) private var auth
  @Dependency(\.mainQueue) private var mainQueue

  let tunnelStore: TunnelStore

  private let logger: AppLogger
  private var cancellables = Set<AnyCancellable>()

  @Published private(set) var loginStatus: LoginStatus {
    didSet {
      self.handleLoginStatusChanged()
    }
  }

  private var status: NEVPNStatus = .invalid

  // Try to automatically reconnect on network changes
  private static let maxReconnectionAttemptCount = 60
  private let reconnectDelaySecs = 1
  private var reconnectionAttemptsRemaining = maxReconnectionAttemptCount

  init(tunnelStore: TunnelStore, logger: AppLogger) {
    self.tunnelStore = tunnelStore
    self.logger = logger
    self.loginStatus = .uninitialized

    Task {
      self.loginStatus = await self.getLoginStatus(from: tunnelStore.tunnelAuthStatus)
    }

    tunnelStore.$tunnelAuthStatus
      .receive(on: mainQueue)
      .sink { [weak self] tunnelAuthStatus in
        guard let self = self else { return }
        logger.log("Tunnel auth status changed to: \(tunnelAuthStatus)")
        self.updateLoginStatus()
      }
      .store(in: &cancellables)

    tunnelStore.$status
      .sink { [weak self] status in
        guard let self = self else { return }
        Task {
          if status == .disconnected {
            self.handleTunnelDisconnectionEvent()
          }
          self.status = status
        }
      }
      .store(in: &cancellables)
  }

  private var authBaseURL: URL {
    if let advancedSettings = self.tunnelStore.advancedSettings(),
      let url = URL(string: advancedSettings.authBaseURLString)
    {
      return url
    }
    return URL(string: AdvancedSettings.defaultValue.authBaseURLString)!
  }

  private func updateLoginStatus() {
    Task {
      logger.log("\(#function): Tunnel auth status is \(self.tunnelStore.tunnelAuthStatus)")
      let tunnelAuthStatus = tunnelStore.tunnelAuthStatus
      let loginStatus = await self.getLoginStatus(from: tunnelAuthStatus)
      if tunnelAuthStatus != self.tunnelStore.tunnelAuthStatus {
        // The tunnel auth status has changed while we were getting the
        // login status, so this login status is not to be used.
        logger.log("\(#function): Ignoring login status \(loginStatus) that's no longer valid.")
        return
      }
      logger.log("\(#function): Corresponding login status is \(loginStatus)")
      await MainActor.run {
        self.loginStatus = loginStatus
      }
    }
  }

  private func getLoginStatus(from tunnelAuthStatus: TunnelAuthStatus?) async -> LoginStatus {
    switch tunnelAuthStatus {
    case nil:
      return .uninitialized
    case .noTunnelFound:
      return .needsTunnelCreationPermission
    case .signedOut:
      return .signedOut
    case .signedIn(let tunnelAuthBaseURL, let tokenReference):
      guard self.authBaseURL == tunnelAuthBaseURL else {
        return .signedOut
      }
      let tunnelBaseURLString = self.authBaseURL.absoluteString
      guard let tokenAttributes = await keychain.loadAttributes(tokenReference),
        tunnelBaseURLString == tokenAttributes.authBaseURLString
      else {
        return .signedOut
      }
      return .signedIn(actorName: tokenAttributes.actorName)
    }
  }

  func signIn() async throws {
    logger.log("\(#function)")

    let authResponse = try await auth.signIn(self.authBaseURL)
    let attributes = Keychain.TokenAttributes(
      authBaseURLString: self.authBaseURL.absoluteString, actorName: authResponse.actorName ?? "")
    let tokenRef = try await keychain.store(authResponse.token, attributes)

    try await tunnelStore.saveAuthStatus(
      .signedIn(authBaseURL: self.authBaseURL, tokenReference: tokenRef))
  }

  func signOut() async {
    logger.log("\(#function)")

    guard case .signedIn = self.tunnelStore.tunnelAuthStatus else {
      logger.log("\(#function): Not signed in, so can't signout.")
      return
    }

    do {
      try await tunnelStore.stop()
      if let tokenRef = try await tunnelStore.signOut() {
        try await keychain.delete(tokenRef)
      }
    } catch {
      logger.error("\(#function): Error signing out: \(error)")
    }
  }

  public func cancelSignIn() {
    auth.cancelSignIn()
  }

  func startTunnel() {
    logger.log("\(#function)")

    guard case .signedIn = self.tunnelStore.tunnelAuthStatus else {
      logger.log("\(#function): Not signed in, so can't start the tunnel.")
      return
    }

    Task {
      do {
        try await tunnelStore.start()
      } catch {
        if case TunnelStoreError.startTunnelErrored(let startTunnelError) = error {
          logger.error(
            "\(#function): Starting tunnel errored: \(String(describing: startTunnelError))"
          )
          handleTunnelDisconnectionEvent()
        } else {
          logger.error("\(#function): Starting tunnel failed: \(String(describing: error))")
          // Disconnection event will be handled in the tunnel status change handler
        }
      }
    }
  }

  func handleTunnelDisconnectionEvent() {
    logger.log("\(#function)")
    if let tsEvent = TunnelShutdownEvent.loadFromDisk(logger: logger) {
      self.logger.log(
        "\(#function): Tunnel shutdown event: \(tsEvent)"
      )
      switch tsEvent.action {
      case .signoutImmediately:
        Task {
          await self.signOut()
        }
        #if os(macOS)
          SessionNotificationHelper.showSignedOutAlertmacOS(logger: self.logger, authStore: self)
        #endif
      case .signoutImmediatelySilently:
        Task {
          await self.signOut()
        }
      case .doNothing:
        break
      }
    } else {
      self.logger.log("\(#function): Tunnel shutdown event not found")
    }
  }

  private func handleLoginStatusChanged() {
    logger.log("\(#function): Login status: \(self.loginStatus)")
    switch self.loginStatus {
    case .signedIn:
      self.startTunnel()
    case .signedOut:
      Task {
        do {
          try await tunnelStore.stop()
        } catch {
          logger.error("\(#function): Error stopping tunnel: \(String(describing: error))")
        }
        if tunnelStore.tunnelAuthStatus != .signedOut {
          // Bring tunnelAuthStatus in line, in case it's out of touch with the login status
          try await tunnelStore.saveAuthStatus(.signedOut)
        }
      }
    case .needsTunnelCreationPermission:
      break
    case .uninitialized:
      break
    }
  }

  func tunnelAuthStatus(for authBaseURL: URL) async -> TunnelAuthStatus {
    if let tokenRef = await keychain.searchByAuthBaseURL(authBaseURL) {
      return .signedIn(authBaseURL: authBaseURL, tokenReference: tokenRef)
    } else {
      return .signedOut
    }
  }
}
