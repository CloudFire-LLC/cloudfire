//
//  FirezoneApp.swift
//  (c) 2024 Firezone, Inc.
//  LICENSE: Apache-2.0
//

import FirezoneKit
import SwiftUI

@main
struct FirezoneApp: App {
  #if os(macOS)
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    @StateObject var askPermissionViewModel: AskPermissionViewModel
  #endif

  #if os(iOS)
    @StateObject var appViewModel: AppViewModel
  #endif

  @StateObject var appStore = AppStore()

  init() {
    let appStore = AppStore()
    self._appStore = StateObject(wrappedValue: appStore)

    #if os(macOS)
      self._askPermissionViewModel =
        StateObject(
          wrappedValue: AskPermissionViewModel(
            tunnelStore: appStore.tunnelStore,
            sessionNotificationHelper: SessionNotificationHelper(
              logger: appStore.logger, tunnelStore: appStore.tunnelStore)
          )
        )
      appDelegate.appStore = appStore
    #elseif os(iOS)
      self._appViewModel =
        StateObject(wrappedValue: AppViewModel(appStore: appStore))
    #endif

  }

  var body: some Scene {
    #if os(iOS)
      WindowGroup {
        AppView(model: appViewModel)
      }
    #else
      WindowGroup(
        "Welcome to Firezone",
        id: AppStore.WindowDefinition.askPermission.identifier
      ) {
        AskPermissionView(model: askPermissionViewModel)
      }
      .handlesExternalEvents(
        matching: [AppStore.WindowDefinition.askPermission.externalEventMatchString]
      )
      WindowGroup(
        "Settings",
        id: AppStore.WindowDefinition.settings.identifier
      ) {
        SettingsView(model: appStore.settingsViewModel)
      }
      .handlesExternalEvents(
        matching: [AppStore.WindowDefinition.settings.externalEventMatchString]
      )
    #endif
  }
}

#if os(macOS)
  @MainActor
  final class AppDelegate: NSObject, NSApplicationDelegate {
    private var isAppLaunched = false
    private var menuBar: MenuBar?

    public var appStore: AppStore?

    func applicationDidFinishLaunching(_: Notification) {
      self.isAppLaunched = true
      if let appStore = self.appStore {
        self.menuBar = MenuBar(
          tunnelStore: appStore.tunnelStore,
          settingsViewModel: appStore.settingsViewModel,
          logger: appStore.logger
        )
      }

      // SwiftUI will show the first window group, so close it on launch
      _ = AppStore.WindowDefinition.allCases.map { $0.window()?.close() }
    }

    func applicationWillTerminate(_: Notification) {
      self.appStore?.tunnelStore.cancelSignIn()
    }
  }
#endif
