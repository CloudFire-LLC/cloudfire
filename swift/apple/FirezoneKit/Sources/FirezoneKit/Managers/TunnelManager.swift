//
//  TunnelManager.swift
//
//
//  Created by Jamil Bou Kheir on 4/2/24.
//
//  Abstracts the nitty gritty of loading and saving to our
//  VPN profile in system preferences.

import CryptoKit
import Foundation
import NetworkExtension

enum TunnelManagerError: Error {
  case cannotSaveIfMissing
}

public enum TunnelManagerKeys {
  static let actorName = "actorName"
  static let authBaseURL = "authBaseURL"
  static let apiURL = "apiURL"
  public static let logFilter = "logFilter"
  public static let disabledResoruces = "disabledResources"
}

public enum TunnelMessage: Codable {
  case getResourceList(Data)
  case signOut
  case setDisabledResources(Set<String>)

  enum CodingKeys: String, CodingKey {
    case type
    case value
  }

  enum MessageType: String, Codable {
    case getResourceList
    case signOut
    case setDisabledResources
  }

  public init(from decoder: Decoder) throws {
      let container = try decoder.container(keyedBy: CodingKeys.self)
      let type = try container.decode(MessageType.self, forKey: .type)
      switch type {
      case .setDisabledResources:
          let value = try container.decode(Set<String>.self, forKey: .value)
          self = .setDisabledResources(value)
      case .getResourceList:
          let value = try container.decode(Data.self, forKey: .value)
          self = .getResourceList(value)
      case .signOut:
          self = .signOut
      }
  }
  public func encode(to encoder: Encoder) throws {
      var container = encoder.container(keyedBy: CodingKeys.self)
      switch self {
      case .setDisabledResources(let value):
          try container.encode(MessageType.setDisabledResources, forKey: .type)
          try container.encode(value, forKey: .value)
      case .getResourceList(let value):
          try container.encode(MessageType.getResourceList, forKey: .type)
          try container.encode(value, forKey: .value)
      case .signOut:
        try container.encode(MessageType.signOut, forKey: .type)
      }
  }
}

public class TunnelManager {
  // Expose closures that someone else can use to respond to events
  // for this manager.
  var statusChangeHandler: ((NEVPNStatus) async -> Void)?

  // Connect status updates with our listeners
  private var tunnelObservingTasks: [Task<Void, Never>] = []

  // Track the "version" of the resource list so we can more efficiently
  // retrieve it from the Provider
  private var resourceListHash = Data()

  // Cache resources on this side of the IPC barrier so we can
  // return them to callers when they haven't changed.
  private var resourcesListCache = Data()

  // Persists our tunnel settings
  private var manager: NETunnelProviderManager?

  // Resources that are currently disabled and will not be used
  public var disabledResources: Set<String> = []

  // Encoder used to send messages to the tunnel
  private let encoder = PropertyListEncoder()

  // Use separate bundle IDs for release and debug.
  // Helps with testing releases and dev builds on the same Mac.
  #if DEBUG
    private let bundleIdentifier = Bundle.main.bundleIdentifier.map {
      "\($0).debug.network-extension"
    }

    private let bundleDescription = "Firezone (Debug)"
  #else
    private let bundleIdentifier = Bundle.main.bundleIdentifier.map { "\($0).network-extension" }
    private let bundleDescription = "Firezone"
  #endif

  init() {
    manager = nil
  }

  // Initialize and save a new VPN profile in system Preferences
  func create() async throws -> Settings {
    let protocolConfiguration = NETunnelProviderProtocol()
    let manager = NETunnelProviderManager()
    let settings = Settings.defaultValue

    protocolConfiguration.providerConfiguration = settings.toProviderConfiguration()
    protocolConfiguration.providerBundleIdentifier = bundleIdentifier
    protocolConfiguration.serverAddress = settings.apiURL
    manager.localizedDescription = bundleDescription
    manager.protocolConfiguration = protocolConfiguration
    encoder.outputFormat = .binary

    // Save the new VPN profile to System Preferences and reload it,
    // which should update our status from invalid -> disconnected.
    // If the user denied the operation, the status will be .invalid
    try await manager.saveToPreferences()
    try await manager.loadFromPreferences()

    await statusChangeHandler?(manager.connection.status)
    self.manager = manager

    return settings
  }

  func load(callback: @escaping (NEVPNStatus, Settings?, String?) -> Void) {
    Task {
      // loadAllFromPreferences() returns list of tunnel configurations created by our main app's bundle ID.
      // Since our bundle ID can change (by us), find the one that's current and ignore the others.
      guard let managers = try? await NETunnelProviderManager.loadAllFromPreferences()
      else {
        Log.app.error("\(#function): Could not load VPN configurations!")
        return
      }

      Log.app.log("\(#function): \(managers.count) tunnel managers found")
      for manager in managers {
        if let protocolConfiguration = manager.protocolConfiguration as? NETunnelProviderProtocol,
           protocolConfiguration.providerBundleIdentifier == bundleIdentifier,
           let providerConfiguration = protocolConfiguration.providerConfiguration as? [String: String]
        {
          // Found it
          let settings = Settings.fromProviderConfiguration(providerConfiguration)
          let actorName = providerConfiguration[TunnelManagerKeys.actorName]
          if let disabledResourcesData = providerConfiguration[TunnelManagerKeys.disabledResoruces]?.data(using: .utf8) {
            self.disabledResources = (try? JSONDecoder().decode(Set<String>.self, from: disabledResourcesData)) ?? Set()

          }
          let status = manager.connection.status

          // Share what we found with our caller
          callback(status, settings, actorName)

          // Update our state
          self.manager = manager

          // Stop looking for our tunnel
          break
        }
      }

      // Hook up status updates
      setupTunnelObservers()

      // If no tunnel configuration was found, update state to
      // prompt user to create one.
      if manager == nil {
        callback(.invalid, nil, nil)
      }
    }
  }

  func saveActorName(_ actorName: String?) async throws {
    guard let manager = manager,
          let protocolConfiguration = manager.protocolConfiguration as? NETunnelProviderProtocol,
          var providerConfiguration = protocolConfiguration.providerConfiguration
    else {
      Log.app.error("Manager doesn't seem initialized. Can't save settings.")
      throw TunnelManagerError.cannotSaveIfMissing
    }

    providerConfiguration[TunnelManagerKeys.actorName] = actorName
    protocolConfiguration.providerConfiguration = providerConfiguration
    manager.protocolConfiguration = protocolConfiguration

    // We always set this to true when starting the tunnel in case our tunnel
    // was disabled by the system for some reason.
    manager.isEnabled = true

    try await manager.saveToPreferences()
    try await manager.loadFromPreferences()
  }

  func saveSettings(_ settings: Settings) async throws {
    guard let manager = manager,
          let protocolConfiguration = manager.protocolConfiguration as? NETunnelProviderProtocol,
          let providerConfiguration = protocolConfiguration.providerConfiguration as? [String: String]
    else {
      Log.app.error("Manager doesn't seem initialized. Can't save settings.")
      throw TunnelManagerError.cannotSaveIfMissing
    }

    var newProviderConfiguration = settings.toProviderConfiguration()

    // Don't clobber existing actorName
    newProviderConfiguration[TunnelManagerKeys.actorName] = providerConfiguration[TunnelManagerKeys.actorName]
    protocolConfiguration.providerConfiguration = newProviderConfiguration
    protocolConfiguration.serverAddress = settings.apiURL
    manager.protocolConfiguration = protocolConfiguration

    // We always set this to true when starting the tunnel in case our tunnel
    // was disabled by the system for some reason.
    manager.isEnabled = true

    Log.app.error("searchme: \(newProviderConfiguration["disabledResources"])")
    try await manager.saveToPreferences()
    try await manager.loadFromPreferences()
  }

  func start(token: String? = nil) {
    var options: [String: NSObject]?

    if let token = token {
      options = ["token": token as NSObject]
    }

    do {
      try session().startTunnel(options: options)
    } catch {
      Log.app.error("Error starting tunnel: \(error)")
    }
  }

  func stop(clearToken: Bool = false) {
    if clearToken {
      do {
        try session().sendProviderMessage(encoder.encode(TunnelMessage.signOut)) { _ in
          self.session().stopTunnel()
        }
      } catch {
        Log.app.error("\(#function): \(error)")
      }
    } else {
      session().stopTunnel()
    }
  }

  func updateDisabledResources() {
    guard session().status == .connected else { return }

    try? session().sendProviderMessage(encoder.encode(TunnelMessage.setDisabledResources(disabledResources))) { _ in }
  }

  func toggleResource(resource: String, enabled: Bool) {
    if enabled {
      disabledResources.remove(resource)
    } else {
      disabledResources.insert(resource)
    }

    updateDisabledResources()
  }

  func fetchResources(callback: @escaping (Data) -> Void) {
    guard session().status == .connected else { return }

    do {
      try session().sendProviderMessage(encoder.encode(TunnelMessage.getResourceList(resourceListHash))) { data in
        if let data = data {
          self.resourceListHash = Data(SHA256.hash(data: data))
          self.resourcesListCache = data
        }

        callback(self.resourcesListCache)
      }
    } catch {
      Log.app.error("Error: sendProviderMessage: \(error)")
    }
  }

  private func session() -> NETunnelProviderSession {
    guard let manager = manager,
          let session = manager.connection as? NETunnelProviderSession
    else { fatalError("Could not cast tunnel connection to NETunnelProviderSession!") }

    return session
  }

  // Subscribe to system notifications about our VPN status changing
  // and let our handler know about them.
  private func setupTunnelObservers() {
    Log.app.log("\(#function)")

    for task in tunnelObservingTasks {
      task.cancel()
    }
    tunnelObservingTasks.removeAll()

    tunnelObservingTasks.append(
      Task {
        for await notification in NotificationCenter.default.notifications(
          named: .NEVPNStatusDidChange
        ) {
          guard let session = notification.object as? NETunnelProviderSession
          else {
            Log.app.error("\(#function): NEVPNStatusDidChange notification doesn't seem to be valid")
            return
          }

          if session.status == .disconnected {
            // Reset resource list on disconnect
            resourceListHash = Data()
            resourcesListCache = Data()
          }

          await statusChangeHandler?(session.status)
        }
      }
    )
  }
}
