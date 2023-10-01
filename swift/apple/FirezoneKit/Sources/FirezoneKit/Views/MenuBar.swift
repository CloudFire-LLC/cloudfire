//
//  MenuBar.swift
//  (c) 2023 Firezone, Inc.
//  LICENSE: Apache-2.0
//

// swiftlint:disable function_parameter_count

#if os(macOS)
import Combine
import Dependencies
import OSLog
import SwiftUI
import NetworkExtension

@MainActor
public final class MenuBar: NSObject {
    let logger = Logger.make(for: MenuBar.self)
    @Dependency(\.mainQueue) private var mainQueue

    private var appStore: AppStore? {
        didSet {
            setupObservers()
        }
    }

    private var cancellables: Set<AnyCancellable> = []
    private var statusItem: NSStatusItem
    private var orderedResources: [DisplayableResources.Resource] = []
    private var isMenuVisible = false {
        didSet { handleMenuVisibilityOrStatusChanged() }
    }
    private lazy var signedOutIcon = NSImage(named: "MenuBarIconSignedOut")
    private lazy var signedInConnectedIcon = NSImage(named: "MenuBarIconSignedInConnected")
    private lazy var signedInNotConnectedIcon = NSImage(named: "MenuBarIconSignedInNotConnected")

    private lazy var connectingAnimationImages = [
        NSImage(named: "MenuBarIconConnecting1"),
        NSImage(named: "MenuBarIconConnecting2"),
        NSImage(named: "MenuBarIconConnecting3")
    ]
    private var connectingAnimationImageIndex: Int = 0
    private var connectingAnimationTimer: Timer?

    let settingsViewModel: SettingsViewModel
    private var loginStatus: AuthStore.LoginStatus = .signedOut(accountId: nil)
    private var tunnelStatus: NEVPNStatus = .invalid

    public init(settingsViewModel: SettingsViewModel) {
        self.settingsViewModel = settingsViewModel

        settingsViewModel.onSettingsSaved = {
            // TODO: close settings window and sign in
        }

        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)

        super.init()
        createMenu()

        if let button = statusItem.button {
            button.image = signedOutIcon
        }

        Task {
            self.appStore = AppStore(tunnelStore: TunnelStore.shared)
            updateStatusItemIcon()
        }
    }

    private func setupObservers() {
        appStore?.auth.$loginStatus
            .receive(on: mainQueue)
            .sink { [weak self] loginStatus in
                self?.loginStatus = loginStatus
                self?.updateStatusItemIcon()
                self?.handleLoginOrTunnelStatusChanged()
            }
            .store(in: &cancellables)

        appStore?.tunnel.$status
            .receive(on: mainQueue)
            .sink { [weak self] status in
                self?.tunnelStatus = status
                self?.updateStatusItemIcon()
                self?.handleLoginOrTunnelStatusChanged()
                self?.handleMenuVisibilityOrStatusChanged()
            }
            .store(in: &cancellables)

        appStore?.tunnel.$resources
            .receive(on: mainQueue)
            .sink { [weak self] resources in
                guard let self = self else { return }
                self.setOrderedResources(resources.orderedResources)
            }
            .store(in: &cancellables)
    }

    private lazy var menu = NSMenu()

    private lazy var signInMenuItem = createMenuItem(
        menu,
        title: "Sign in",
        action: #selector(signInButtonTapped),
        target: self
    )
    private lazy var signOutMenuItem = createMenuItem(
        menu,
        title: "Sign out",
        action: #selector(signOutButtonTapped),
        isHidden: true,
        target: self
    )
    private lazy var resourcesTitleMenuItem = createMenuItem(
        menu,
        title: "No Resources",
        action: nil,
        isHidden: true,
        target: self
    )
    private lazy var resourcesUnavailableMenuItem = createMenuItem(
        menu,
        title: "Resources unavailable",
        action: nil,
        isHidden: true,
        target: self
    )
    private lazy var resourcesUnavailableReasonMenuItem = createMenuItem(
        menu,
        title: "",
        action: #selector(reconnectButtonTapped),
        isHidden: true,
        target: self
    )
    private lazy var resourcesSeparatorMenuItem = NSMenuItem.separator()
    private lazy var aboutMenuItem: NSMenuItem = {
        let menuItem = createMenuItem(
            menu,
            title: "About",
            action: #selector(aboutButtonTapped),
            target: self
        )
        if let appName = Bundle.main.infoDictionary?[kCFBundleNameKey as String] as? String {
            menuItem.title = "About \(appName)"
        }
        return menuItem
    }()
    private lazy var settingsMenuItem = createMenuItem(
        menu,
        title: "Settings",
        action: #selector(settingsButtonTapped),
        target: self
    )
    private lazy var quitMenuItem: NSMenuItem = {
        let menuItem = createMenuItem(
            menu,
            title: "Quit",
            action: #selector(NSApplication.terminate(_:)),
            key: "q",
            target: nil
        )
        if let appName = Bundle.main.infoDictionary?[kCFBundleNameKey as String] as? String {
            menuItem.title = "Quit \(appName)"
        }
        return menuItem
    }()

    private func createMenu() {
        menu.addItem(signInMenuItem)
        menu.addItem(signOutMenuItem)
        menu.addItem(NSMenuItem.separator())

        menu.addItem(resourcesTitleMenuItem)
        menu.addItem(resourcesUnavailableMenuItem)
        menu.addItem(resourcesUnavailableReasonMenuItem)
        menu.addItem(resourcesSeparatorMenuItem)

        menu.addItem(aboutMenuItem)
        menu.addItem(settingsMenuItem)
        menu.addItem(quitMenuItem)

        menu.delegate = self

        statusItem.menu = menu
    }

    private func createMenuItem(
        _: NSMenu,
        title: String,
        action: Selector?,
        isHidden: Bool = false,
        key: String = "",
        target: AnyObject?
    ) -> NSMenuItem {
        let item = NSMenuItem(title: title, action: action, keyEquivalent: key)

        item.isHidden = isHidden
        item.target = target
        item.isEnabled = (action != nil)

        return item
    }

    @objc private func reconnectButtonTapped() {
        Task {
            if case .signedIn = appStore?.auth.loginStatus {
                do {
                    try await appStore?.tunnel.start()
                } catch {
                    logger.error("error connecting to tunnel (reconnect): \(String(describing: error))")
                }
            }
        }
    }

    @objc private func signInButtonTapped() {
        Task {
            do {
                try await appStore?.auth.signIn()
            } catch FirezoneError.missingTeamId {
                openSettingsWindow()
            } catch {
                logger.error("Error signing in: \(String(describing: error))")
            }
        }
    }

    @objc private func signOutButtonTapped() {
        Task {
            do {
                try await appStore?.auth.signOut()
            } catch {
                logger.error("error signing out: \(String(describing: error))")
            }
        }
    }

    @objc private func settingsButtonTapped() {
        openSettingsWindow()
    }

    @objc private func aboutButtonTapped() {
        NSApp.activate(ignoringOtherApps: true)
        NSApp.orderFrontStandardAboutPanel(self)
    }

    private func openSettingsWindow() {
        NSWorkspace.shared.open(URL(string: "firezone://settings")!)
    }

    private func updateStatusItemIcon() {
        self.statusItem.button?.image = {
            switch self.loginStatus {
            case .signedOut, .uninitialized:
                return self.signedOutIcon
            case .signedIn:
                switch self.tunnelStatus {
                case .invalid, .disconnected, .reasserting:
                    return self.signedInNotConnectedIcon
                case .connected:
                    return self.signedInConnectedIcon
                case .connecting, .disconnecting:
                    return self.connectingAnimationImages.last!
                @unknown default:
                    return nil
                }
            }
        }()
        if self.tunnelStatus == .connecting || self.tunnelStatus == .disconnecting {
            self.startConnectingAnimation()
        } else {
            self.stopConnectingAnimation()
        }
    }

    private func startConnectingAnimation() {
        guard connectingAnimationTimer == nil else { return }
        let timer = Timer(timeInterval: 0.40, repeats: true) { [weak self] _ in
            guard let self = self else { return }
            Task {
                await self.connectingAnimationShowNextFrame()
            }
        }
        RunLoop.main.add(timer, forMode: .common)
        self.connectingAnimationTimer = timer
    }

    private func stopConnectingAnimation() {
        guard let timer = self.connectingAnimationTimer else { return }
        timer.invalidate()
        connectingAnimationTimer = nil
        connectingAnimationImageIndex = 0
    }

    private func connectingAnimationShowNextFrame() async {
        self.statusItem.button?.image = self.connectingAnimationImages[self.connectingAnimationImageIndex]
        self.connectingAnimationImageIndex = (self.connectingAnimationImageIndex + 1) % self.connectingAnimationImages.count
    }

    private func handleLoginOrTunnelStatusChanged() {
        // Update "Sign In" / "Sign Out" menu items
        switch self.loginStatus {
        case .uninitialized:
            signInMenuItem.title = "Initializing"
            signInMenuItem.target = nil
            signOutMenuItem.isHidden = true
        case .signedOut:
            signInMenuItem.title = "Sign In"
            signInMenuItem.target = self
            signOutMenuItem.isHidden = true
        case .signedIn(_, let actorName):
            signInMenuItem.title = actorName.isEmpty ? "Signed in" : "Signed in as \(actorName)"
            signInMenuItem.target = nil
            signOutMenuItem.isHidden = false
        }
        // Update resources "header" menu items
        switch (self.loginStatus, self.tunnelStatus) {
        case (.uninitialized, _):
            resourcesTitleMenuItem.isHidden = true
            resourcesUnavailableMenuItem.isHidden = true
            resourcesUnavailableReasonMenuItem.isHidden = true
            resourcesSeparatorMenuItem.isHidden = true
        case (.signedOut, _):
            resourcesTitleMenuItem.isHidden = true
            resourcesUnavailableMenuItem.isHidden = true
            resourcesUnavailableReasonMenuItem.isHidden = true
            resourcesSeparatorMenuItem.isHidden = true
        case (.signedIn, .connecting):
            resourcesTitleMenuItem.isHidden = true
            resourcesUnavailableMenuItem.isHidden = false
            resourcesUnavailableReasonMenuItem.isHidden = false
            resourcesUnavailableReasonMenuItem.target = nil
            resourcesUnavailableReasonMenuItem.title = "Connecting…"
            resourcesSeparatorMenuItem.isHidden = false
        case (.signedIn, .connected):
            resourcesTitleMenuItem.isHidden = false
            resourcesUnavailableMenuItem.isHidden = true
            resourcesUnavailableReasonMenuItem.isHidden = true
            resourcesTitleMenuItem.title = "Resources"
            resourcesSeparatorMenuItem.isHidden = false
        case (.signedIn, .reasserting):
            resourcesTitleMenuItem.isHidden = true
            resourcesUnavailableMenuItem.isHidden = false
            resourcesUnavailableReasonMenuItem.isHidden = false
            resourcesUnavailableReasonMenuItem.target = nil
            resourcesUnavailableReasonMenuItem.title = "No network connectivity"
            resourcesSeparatorMenuItem.isHidden = false
        case (.signedIn, .disconnecting):
            resourcesTitleMenuItem.isHidden = true
            resourcesUnavailableMenuItem.isHidden = false
            resourcesUnavailableReasonMenuItem.isHidden = false
            resourcesUnavailableReasonMenuItem.target = nil
            resourcesUnavailableReasonMenuItem.title = "Disconnecting…"
            resourcesSeparatorMenuItem.isHidden = false
        case (.signedIn, _):
            // Ideally, this shouldn't happen, but it's better
            // we handle this case, so that in case connlib errors out,
            // the user is able to try to reconnect.
            resourcesTitleMenuItem.isHidden = true
            resourcesUnavailableMenuItem.isHidden = false
            resourcesUnavailableReasonMenuItem.isHidden = false
            resourcesUnavailableReasonMenuItem.target = self
            resourcesUnavailableReasonMenuItem.isEnabled = true
            resourcesUnavailableReasonMenuItem.title = "Reconnect"
            resourcesSeparatorMenuItem.isHidden = false
        }
    }

    private func handleMenuVisibilityOrStatusChanged() {
        guard let appStore = appStore else { return }
        let status = appStore.tunnel.status
        if isMenuVisible && status == .connected {
            appStore.tunnel.beginUpdatingResources()
        } else {
            appStore.tunnel.endUpdatingResources()
        }
    }

    private func setOrderedResources(_ newOrderedResources: [DisplayableResources.Resource]) {
        if resourcesTitleMenuItem.isHidden && resourcesSeparatorMenuItem.isHidden {
            guard newOrderedResources.isEmpty else {
                return
            }
        }
        let diff = newOrderedResources.difference(
            from: self.orderedResources,
            by: { $0.name == $1.name && $0.location == $1.location }
        )
        let baseIndex = menu.index(of: resourcesTitleMenuItem) + 1
        for change in diff {
            switch change {
            case .insert(offset: let offset, element: let element, associatedWith: _):
                let menuItem = createResourceMenuItem(title: element.name, submenuTitle: element.location)
                menu.insertItem(menuItem, at: baseIndex + offset)
                orderedResources.insert(element, at: offset)
            case .remove(offset: let offset, element: _, associatedWith: _):
                menu.removeItem(at: baseIndex + offset)
                orderedResources.remove(at: offset)
            }
        }
        resourcesTitleMenuItem.title = orderedResources.isEmpty ? "No Resources" : "Resources"
    }

    private func createResourceMenuItem(title: String, submenuTitle: String) -> NSMenuItem {
        let item = NSMenuItem(title: title, action: nil, keyEquivalent: "")

        let subMenu = NSMenu()
        let subMenuItem = NSMenuItem(title: submenuTitle, action: #selector(resourceValueTapped(_:)), keyEquivalent: "")
        subMenuItem.isEnabled = true
        subMenuItem.target = self
        subMenu.addItem(subMenuItem)

        item.isHidden = false
        item.submenu = subMenu

        return item
    }

    @objc private func resourceValueTapped(_ sender: AnyObject?) {
        if let value = (sender as? NSMenuItem)?.title {
            copyToClipboard(value)
        }
    }

    private func copyToClipboard(_ string: String) {
        let pasteBoard = NSPasteboard.general
        pasteBoard.clearContents()
        pasteBoard.writeObjects([string as NSString])
    }
}

extension MenuBar: NSMenuDelegate {
    public func menuNeedsUpdate(_ menu: NSMenu) {
        isMenuVisible = true
    }
    public func menuDidClose(_ menu: NSMenu) {
        isMenuVisible = false
    }
}
#endif
