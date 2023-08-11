//
//  TunnelStore.swift
//  (c) 2023 Firezone, Inc.
//  LICENSE: Apache-2.0
//

import Combine
import Foundation
import NetworkExtension
import OSLog

enum TunnelStoreError: Error {
    case tunnelCouldNotBeStarted
}

final class TunnelStore: ObservableObject {
  private static let logger = Logger.make(for: TunnelStore.self)

  var tunnel: NETunnelProviderManager {
    didSet { setupTunnelObservers() }
  }

  @Published private(set) var status: NEVPNStatus = .invalid {
    didSet { TunnelStore.logger.info("status changed: \(self.status.description)") }
  }

  @Published private(set) var isEnabled = false {
    didSet { TunnelStore.logger.info("isEnabled changed: \(self.isEnabled.description)") }
  }

  @Published private(set) var resources = DisplayableResources()

  private var resourcesTimer: Timer? {
    didSet(oldValue) { oldValue?.invalidate() }
  }

  private let controlPlaneURL: URL
  private var tunnelObservingTasks: [Task<Void, Never>] = []
  private var startTunnelContinuation: CheckedContinuation<(), Error>?

  init(tunnel: NETunnelProviderManager) {
    self.controlPlaneURL = Self.getControlPlaneURLFromInfoPlist()
    self.tunnel = tunnel
    tunnel.isEnabled = true
    setupTunnelObservers()
  }

  static func loadOrCreate() async throws -> NETunnelProviderManager {
    logger.trace("\(#function)")

    let managers = try await NETunnelProviderManager.loadAllFromPreferences()

    if let tunnel = managers.first {
      return tunnel
    }

    let tunnel = makeManager()
    try await tunnel.saveToPreferences()
    try await tunnel.loadFromPreferences()

    return tunnel
  }

  func start(authResponse: AuthResponse) async throws {
    TunnelStore.logger.trace("\(#function)")

    // make sure we have latest preferences before starting
    try await tunnel.loadFromPreferences()

    tunnel.protocolConfiguration = Self.makeProtocolConfiguration(
      controlPlaneURL: self.controlPlaneURL,
      token: authResponse.token
    )
    tunnel.isEnabled = true
    try await tunnel.saveToPreferences()

    let session = tunnel.connection as! NETunnelProviderSession
    try session.startTunnel()
    try await withCheckedThrowingContinuation { continuation in
      self.startTunnelContinuation = continuation
    }
  }

  func stop() {
    TunnelStore.logger.trace("\(#function)")
    let session = tunnel.connection as! NETunnelProviderSession
    session.stopTunnel()
  }

  func beginUpdatingResources() {
    self.updateResources()
    let timer = Timer(timeInterval: 1 /*second*/, repeats: true) { [weak self] _ in
      guard let self = self else { return }
      guard self.status == .connected else { return }
      self.updateResources()
    }
    RunLoop.main.add(timer, forMode: .common)
    self.resourcesTimer = timer
  }

  func endUpdatingResources() {
    self.resourcesTimer = nil
  }

  private func updateResources() {
    let session = tunnel.connection as! NETunnelProviderSession
    let resourcesQuery = resources.versionStringToData()
    do {
      try session.sendProviderMessage(resourcesQuery) { [weak self] reply in
        if let reply = reply { // If reply is nil, then the resources have not changed
          if let updatedResources = DisplayableResources(from: reply) {
            self?.resources = updatedResources
          }
        }
      }
    } catch {
      TunnelStore.logger.error("Error: sendProviderMessage: \(error)")
    }
  }

  private static func makeManager() -> NETunnelProviderManager {
    logger.trace("\(#function)")

    let manager = NETunnelProviderManager()
    manager.localizedDescription = "Firezone"

    let proto = makeProtocolConfiguration()
    manager.protocolConfiguration = proto
    manager.isEnabled = true

    return manager
  }

  static func getControlPlaneURLFromInfoPlist() -> URL {
    let infoPlistDictionary = Bundle.main.infoDictionary
    guard let urlScheme = (infoPlistDictionary?["ControlPlaneURLScheme"] as? String), !urlScheme.isEmpty else {
      fatalError("AuthURLScheme missing in Info.plist. Please define AUTH_URL_SCHEME, AUTH_URL_HOST, CONTROL_PLANE_URL_SCHEME, and CONTROL_PLANE_URL_HOST in Server.xcconfig.")
    }
    guard let urlHost = (infoPlistDictionary?["ControlPlaneURLHost"] as? String), !urlHost.isEmpty else {
      fatalError("AuthURLHost missing in Info.plist. Please define AUTH_URL_SCHEME, AUTH_URL_HOST, CONTROL_PLANE_URL_SCHEME, and CONTROL_PLANE_URL_HOST in Server.xcconfig.")
    }
    let urlString = "\(urlScheme)://\(urlHost)/"
    guard let url = URL(string: urlString) else {
      fatalError("Cannot form valid URL from string: \(urlString)")
    }
    return url
  }

  private static func makeProtocolConfiguration(controlPlaneURL: URL? = nil, token: String? = nil) -> NETunnelProviderProtocol {
    let proto = NETunnelProviderProtocol()

    proto.providerBundleIdentifier = Bundle.main.bundleIdentifier.map {
      "\($0).network-extension"
    }
    if let controlPlaneURL = controlPlaneURL, let token = token {
      proto.providerConfiguration = [
        "controlPlaneURL": controlPlaneURL.absoluteString,
        "token": token
      ]
    }
    proto.serverAddress = "Firezone addresses"
    return proto
  }

  private func setupTunnelObservers() {
    TunnelStore.logger.trace("\(#function)")

    tunnelObservingTasks.forEach { $0.cancel() }
    tunnelObservingTasks.removeAll()

    tunnelObservingTasks.append(
      Task {
        for await notification in NotificationCenter.default.notifications(
          named: .NEVPNStatusDidChange,
          object: nil
        ) {
          guard let session = notification.object as? NETunnelProviderSession,
                let tunnelProvider = session.manager as? NETunnelProviderManager
          else {
            return
          }
          self.status = tunnelProvider.connection.status
          if let startTunnelContinuation = self.startTunnelContinuation {
            switch self.status {
              case .connected:
                startTunnelContinuation.resume(returning: ())
                self.startTunnelContinuation = nil
              case .disconnected:
                startTunnelContinuation.resume(throwing: TunnelStoreError.tunnelCouldNotBeStarted)
                self.startTunnelContinuation = nil
              default:
                break
            }
          }
        }
      }
    )
  }

  func removeProfile() async throws {
    TunnelStore.logger.trace("\(#function)")

    try await tunnel.removeFromPreferences()
  }
}

// MARK: - Extensions

/// Make NEVPNStatus convertible to a string
extension NEVPNStatus: CustomStringConvertible {
  public var description: String {
    switch self {
    case .disconnected: return "Disconnected"
    case .invalid: return "Invalid"
    case .connected: return "Connected"
    case .connecting: return "Connecting"
    case .disconnecting: return "Disconnecting"
    case .reasserting: return "Reconnecting"
    @unknown default: return "Unknown"
    }
  }
}
