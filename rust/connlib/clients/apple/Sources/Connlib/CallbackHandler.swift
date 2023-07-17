//
//  CallbackHandler.swift
//

import NetworkExtension
import os.log

// When the FFI changes from the Rust side, change the CallbackHandler
// functions along with that, but not the delegate protocol.
// When the app gets updated to use the FFI, the delegate protocol
// shall get updated.
// This is so that the app stays buildable even when the FFI changes.

// TODO: https://github.com/chinedufn/swift-bridge/issues/150
extension SwiftConnlibError: @unchecked Sendable {}
extension SwiftConnlibError: Error {}

public protocol CallbackHandlerDelegate: AnyObject {
  func onConnect(tunnelAddressIPv4: String, tunnelAddressIPv6: String)
  func onUpdateResources(resourceList: String)
  func onDisconnect()
  func onError(error: Error, isRecoverable: Bool)
}

public class CallbackHandler {
  public weak var delegate: CallbackHandlerDelegate?
  private let logger = Logger(subsystem: "dev.firezone.firezone", category: "callbackhandler")

  func onSetInterfaceConfig(tunnelAddresses: TunnelAddresses, dnsAddress: RustString) {
    logger.debug("CallbackHandler.onSetInterfaceConfig: IPv4: \(tunnelAddresses.address4.toString(), privacy: .public), IPv6: \(tunnelAddresses.address6.toString(), privacy: .public), DNS: \(dnsAddress.toString(), privacy: .public)")
    // Unimplemented
  }

  func onTunnelReady() {
    logger.debug("CallbackHandler.onTunnelReady")
    // Unimplemented
  }

  func onAddRoute(route: RustString) {
    logger.debug("CallbackHandler.onAddRoute: \(route.toString(), privacy: .public)")
    // Unimplemented
  }

  func onRemoveRoute(route: RustString) {
    logger.debug("CallbackHandler.onRemoveRoute: \(route.toString(), privacy: .public)")
    // Unimplemented
  }

  func onUpdateResources(resourceList: ResourceList) {
    logger.debug("CallbackHandler.onUpdateResources: \(resourceList.resources.toString(), privacy: .public)")
    delegate?.onUpdateResources(resourceList: resourceList.resources.toString())
  }

  func onDisconnect() {
    logger.debug("CallbackHandler.onDisconnect")
    delegate?.onDisconnect()
  }

  func onError(error: SwiftConnlibError, error_type: SwiftErrorType) {
    logger.debug("CallbackHandler.onError: \(error, privacy: .public) (\(error_type == .Recoverable ? "Recoverable" : "Fatal", privacy: .public)")
    delegate?.onError(error: error, isRecoverable: error_type == .Recoverable)
  }
}
