//
//  NetworkSettings.swift
//  (c) 2023 Firezone, Inc.
//  LICENSE: Apache-2.0

import Foundation
import NetworkExtension
import os.log

class NetworkSettings {

  enum DNSFallbackStrategy: String {
    // How to handle DNS requests for domains not handled by Firezone
    case systemResolver = "system_resolver" // Have the OS handle it using split-DNS
    case upstreamResolver = "upstream_resolver" // Have connlib pass it on to a user-specified DNS server

    init(_ string: String) {
      if string == "upstream_resolver" {
        self = .upstreamResolver
      } else if string == "system_resolver" {
        self = .systemResolver
      } else {
        // silent default
        self = .systemResolver
      }
    }
  }

  // Unchanging values
  let tunnelAddressIPv4: String
  let tunnelAddressIPv6: String
  let dnsAddress: String

  // Modifiable values
  private(set) var dnsFallbackStrategy: DNSFallbackStrategy
  private(set) var routes: [String] = []
  private(set) var resourceDomains: [String] = []

  // To keep track of modifications
  private(set) var hasUnappliedChanges: Bool

  init(tunnelAddressIPv4: String, tunnelAddressIPv6: String, dnsAddress: String, dnsFallbackStrategy: DNSFallbackStrategy) {
    self.tunnelAddressIPv4 = tunnelAddressIPv4
    self.tunnelAddressIPv6 = tunnelAddressIPv6
    self.dnsAddress = dnsAddress
    self.dnsFallbackStrategy = dnsFallbackStrategy
    self.hasUnappliedChanges = true
  }

  func setDNSFallbackStrategy(_ dnsFallbackStrategy: DNSFallbackStrategy) {
    if self.dnsFallbackStrategy != dnsFallbackStrategy {
      self.dnsFallbackStrategy = dnsFallbackStrategy
      self.hasUnappliedChanges = true
    }
  }

  func addRoute(_ route: String) {
    if !self.routes.contains(route) {
      self.routes.append(route)
      self.hasUnappliedChanges = true
    }
  }

  func removeRoute(_ route: String) {
    if self.routes.contains(route) {
      self.routes.removeAll(where: { $0 == route })
      self.hasUnappliedChanges = true
    }
  }

  func setResourceDomains(_ resourceDomains: [String]) {
    let sortedResourceDomains = resourceDomains.sorted()
    if self.resourceDomains != sortedResourceDomains {
      self.resourceDomains = sortedResourceDomains
      if dnsFallbackStrategy == .systemResolver {
        self.hasUnappliedChanges = true
      }
    }
  }

  func apply(on packetTunnelProvider: NEPacketTunnelProvider, logger: Logger, completionHandler: ((Error?) -> Void)?) {

    guard self.hasUnappliedChanges else {
      logger.error("NetworkSettings.apply: No changes to apply")
      completionHandler?(nil)
      return
    }

    logger.debug("NetworkSettings.apply: Applying network settings")

    var tunnelIPv4Routes: [NEIPv4Route] = []
    var tunnelIPv6Routes: [NEIPv6Route] = []

    for route in routes {
      let components = route.split(separator: "/")
      guard components.count == 2 else {
        logger.error("NetworkSettings.apply: Ignoring invalid route '\(route, privacy: .public)'")
        continue
      }
      let address = String(components[0])
      let networkPrefixLengthString = String(components[1])
      if let groupSeparator = address.first(where: { $0 == "." || $0 == ":"}) {
        if groupSeparator == "." { // IPv4 address
          if IPv4Address(address) == nil {
            logger.error("NetworkSettings.apply: Ignoring invalid IPv4 address '\(address, privacy: .public)'")
            continue
          }
          let validNetworkPrefixLength = Self.validNetworkPrefixLength(fromString: networkPrefixLengthString, maximumValue: 32)
          let ipv4SubnetMask = Self.ipv4SubnetMask(networkPrefixLength: validNetworkPrefixLength)
          logger.debug("NetworkSettings.apply: Adding IPv4 route: \(address) (subnet mask: \(ipv4SubnetMask))")
          tunnelIPv4Routes.append(NEIPv4Route(destinationAddress: address, subnetMask: ipv4SubnetMask))
        }
        if groupSeparator == ":" { // IPv6 address
          if IPv6Address(address) == nil {
            logger.error("NetworkSettings.apply: Ignoring invalid IPv6 address '\(address, privacy: .public)'")
            continue
          }
          let validNetworkPrefixLength = Self.validNetworkPrefixLength(fromString: networkPrefixLengthString, maximumValue: 128)
          logger.debug("NetworkSettings.apply: Adding IPv6 route: \(address) (prefix length: \(validNetworkPrefixLength))")
          tunnelIPv6Routes.append(NEIPv6Route(destinationAddress: address, networkPrefixLength: NSNumber(integerLiteral: validNetworkPrefixLength)))
        }
      } else {
        logger.error("NetworkSettings.apply: Ignoring invalid route '\(route, privacy: .public)'")
      }
    }

    // We don't really know the connlib gateway IP address at this point, but just using 127.0.0.1 is okay
    // because the OS doesn't really need this IP address.
    // NEPacketTunnelNetworkSettings taking in tunnelRemoteAddress is probably a bad abstraction caused by
    // NEPacketTunnelNetworkSettings inheriting from NETunnelNetworkSettings.

    let tunnelNetworkSettings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "127.0.0.1")

    let ipv4Settings = NEIPv4Settings(addresses: [tunnelAddressIPv4], subnetMasks: ["255.255.255.255"])
    ipv4Settings.includedRoutes = tunnelIPv4Routes
    tunnelNetworkSettings.ipv4Settings = ipv4Settings

    let ipv6Settings = NEIPv6Settings(addresses: [tunnelAddressIPv6], networkPrefixLengths: [128])
    ipv6Settings.includedRoutes = tunnelIPv6Routes
    tunnelNetworkSettings.ipv6Settings = ipv6Settings

    let dnsSettings = NEDNSSettings(servers: [dnsAddress])
    switch dnsFallbackStrategy {
      case .systemResolver:
        // Enable split-DNS. Only those domains matching the resources will be sent to the tunnel's DNS.
        dnsSettings.matchDomains = resourceDomains
      case .upstreamResolver:
        // All DNS queries go to the tunnel's DNS.
        dnsSettings.matchDomains = [""]
    }
    tunnelNetworkSettings.dnsSettings = dnsSettings

    self.hasUnappliedChanges = false
    packetTunnelProvider.setTunnelNetworkSettings(tunnelNetworkSettings) { error in
      if let error = error {
        logger.error("NetworkSettings.apply: Error: \(error, privacy: .public)")
      } else {
        guard !self.hasUnappliedChanges else {
          // Changes were made while the packetTunnelProvider was setting the network settings
          logger.debug("NetworkSettings.apply: Applying changes made to network settings while we were applying the network settings")
          self.apply(on: packetTunnelProvider, logger: logger, completionHandler: completionHandler)
          return
        }
        logger.debug("NetworkSettings.apply: Applied successfully")
      }
      completionHandler?(error)
    }
  }
}

private extension NetworkSettings {
  private static func validNetworkPrefixLength(fromString string: String, maximumValue: Int) -> Int {
    guard let networkPrefixLength = Int(string) else { return 0 }
    if networkPrefixLength < 0 { return 0 }
    if networkPrefixLength > maximumValue { return maximumValue }
    return networkPrefixLength
  }

  private static func ipv4SubnetMask(networkPrefixLength: Int) -> String {
    precondition(networkPrefixLength >= 0 && networkPrefixLength <= 32)
    let mask: UInt32 = 0xFFFFFFFF
    let maxPrefixLength = 32
    let octets = 4

    let subnetMask = mask & (mask << (maxPrefixLength - networkPrefixLength))
    var parts: [String] = []
    for idx in 0...(octets - 1) {
      let part = String(UInt32(0x000000FF) & (subnetMask >> ((octets - 1 - idx) * 8)), radix: 10)
      parts.append(part)
    }

    return parts.joined(separator: ".")
  }
}
