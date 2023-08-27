//
//  AuthStore.swift
//  (c) 2023 Firezone, Inc.
//  LICENSE: Apache-2.0
//

import Combine
import Dependencies
import Foundation
import OSLog

extension AuthStore: DependencyKey {
  static var liveValue: AuthStore = .shared
}

extension DependencyValues {
  var authStore: AuthStore {
    get { self[AuthStore.self] }
    set { self[AuthStore.self] = newValue }
  }
}

@MainActor
final class AuthStore: ObservableObject {
  private let logger = Logger.make(for: AuthStore.self)

  static let shared = AuthStore()

  enum LoginStatus {
    case uninitialized
    case signedOut
    case signedIn(AuthResponse)
  }

  @Dependency(\.keychain) private var keychain
  @Dependency(\.auth) private var auth
  @Dependency(\.settingsClient) private var settingsClient

  private let authBaseURL: URL
  private var cancellables = Set<AnyCancellable>()

  @Published private(set) var loginStatus: LoginStatus

  private init() {
    self.authBaseURL = Self.getAuthBaseURLFromInfoPlist()
    self.loginStatus = .uninitialized
    Task {
      self.loginStatus = await { () -> LoginStatus in
        guard let teamId = settingsClient.fetchSettings()?.teamId else {
          logger.debug("No team-id found in settings")
          return .signedOut
        }
        guard let token = try? await keychain.token() else {
          logger.debug("Token not found in keychain")
          return .signedOut
        }
        guard let actorName = try? await keychain.actorName() else {
          logger.debug("Actor not found in keychain")
          return .signedOut
        }
        let portalURL = self.authURL(teamId: teamId)
        let authResponse = AuthResponse(portalURL: portalURL, token: token, actorName: actorName)
        logger.debug("Token recovered from keychain.")
        return .signedIn(authResponse)
      }()
    }

    $loginStatus
      .sink { [weak self] loginStatus in
        Task { [weak self] in
          switch loginStatus {
            case .signedIn(let authResponse):
              try? await self?.keychain.save(token: authResponse.token, actorName: authResponse.actorName)
              self?.logger.debug("authResponse saved on keychain.")
            case .signedOut:
              try? await self?.keychain.deleteAuthResponse()
              self?.logger.debug("token deleted from keychain.")
            case .uninitialized:
              break
          }
        }
      }
      .store(in: &cancellables)
  }

  func signIn(teamId: String) async throws {
    logger.trace("\(#function)")

    let portalURL = authURL(teamId: teamId)
    let authResponse = try await auth.signIn(portalURL)
    self.loginStatus = .signedIn(authResponse)
  }

  func signIn() async throws {
    logger.trace("\(#function)")

    guard let teamId = settingsClient.fetchSettings()?.teamId, !teamId.isEmpty else {
      logger.debug("No team-id found in settings")
      throw FirezoneError.missingTeamId
    }

    try await signIn(teamId: teamId)
  }

  func signOut() {
    logger.trace("\(#function)")

    loginStatus = .signedOut
  }

  static func getAuthBaseURLFromInfoPlist() -> URL {
    let infoPlistDictionary = Bundle.main.infoDictionary
    guard let urlScheme = (infoPlistDictionary?["AuthURLScheme"] as? String), !urlScheme.isEmpty else {
      fatalError("AuthURLScheme missing in Info.plist. Please define AUTH_URL_SCHEME, AUTH_URL_HOST, CONTROL_PLANE_URL_SCHEME, and CONTROL_PLANE_URL_HOST in Server.xcconfig.")
    }
    guard let urlHost = (infoPlistDictionary?["AuthURLHost"] as? String), !urlHost.isEmpty else {
      fatalError("AuthURLHost missing in Info.plist. Please define AUTH_URL_SCHEME, AUTH_URL_HOST, CONTROL_PLANE_URL_SCHEME, and CONTROL_PLANE_URL_HOST in Server.xcconfig.")
    }
    let urlString = "\(urlScheme)://\(urlHost)/"
    guard let url = URL(string: urlString) else {
      fatalError("Cannot form valid URL from string: \(urlString)")
    }
    return url
  }

  func authURL(teamId: String) -> URL {
    self.authBaseURL.appendingPathComponent(teamId)
  }
}
