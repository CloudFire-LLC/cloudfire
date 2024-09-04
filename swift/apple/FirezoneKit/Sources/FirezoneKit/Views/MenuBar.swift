//
//  MenuBar.swift
//
//
//  Created by Jamil Bou Kheir on 4/2/24.
//

import Foundation
import Combine
import NetworkExtension
import OSLog
import SwiftUI


#if os(macOS)
@MainActor
// TODO: Refactor to MenuBarExtra for macOS 13+
// https://developer.apple.com/documentation/swiftui/menubarextra
public final class MenuBar: NSObject, ObservableObject {
  private var statusItem: NSStatusItem

  // Wish these could be `[String]` but diffing between different types is tricky
  private var lastShownFavorites: [Resource] = []
  private var lastShownOthers: [Resource] = []
  private var cancellables: Set<AnyCancellable> = []

  @ObservedObject var model: SessionViewModel

  private lazy var signedOutIcon = NSImage(named: "MenuBarIconSignedOut")
  private lazy var signedInConnectedIcon = NSImage(named: "MenuBarIconSignedInConnected")

  private lazy var connectingAnimationImages = [
    NSImage(named: "MenuBarIconConnecting1"),
    NSImage(named: "MenuBarIconConnecting2"),
    NSImage(named: "MenuBarIconConnecting3"),
  ]
  private var connectingAnimationImageIndex: Int = 0
  private var connectingAnimationTimer: Timer?

  public init(model: SessionViewModel) {
    statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
    self.model = model

    super.init()

    if let button = statusItem.button {
      button.image = signedOutIcon
    }

    createMenu()
    setupObservers()
  }

  func showMenu() {
    statusItem.button?.performClick(nil)
  }

  private func setupObservers() {
    model.favorites.$ids
      .receive(on: DispatchQueue.main)
      .sink(receiveValue: { [weak self] ids in
        guard let self = self else { return }
        // When the user clicks to add or remove a favorite, the menu will close anyway, so just recreate the whole menu.
        // This avoids complex logic when changing in and out of the "nothing is favorited" special case
        self.populateResourceMenus([])
        self.populateResourceMenus(model.resources.asArray())
      }).store(in: &cancellables)

    model.$resources
      .receive(on: DispatchQueue.main)
      .sink(receiveValue: { [weak self] resources in
        guard let self = self else { return }
        self.populateResourceMenus(model.resources.asArray())
        self.handleTunnelStatusOrResourcesChanged()
      }).store(in: &cancellables)

    model.$status
      .receive(on: DispatchQueue.main)
      .sink(receiveValue: { [weak self] status in
        guard let self = self else { return }
        self.updateStatusItemIcon(status: model.status)
      }).store(in: &cancellables)
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
    title: "Loading Resources...",
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
    action: nil,
    isHidden: true,
    target: self
  )
  private lazy var resourcesSeparatorMenuItem = NSMenuItem.separator()
  private lazy var otherResourcesMenu: NSMenu = NSMenu()
  private lazy var otherResourcesMenuItem: NSMenuItem = {
    let menuItem = NSMenuItem(title: "Other Resources", action: nil, keyEquivalent: "")
    menuItem.submenu = otherResourcesMenu
    return menuItem
  }()
  private lazy var otherResourcesSeparatorMenuItem = NSMenuItem.separator()
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
  private lazy var adminPortalMenuItem: NSMenuItem = {
    let menuItem = createMenuItem(
      menu,
      title: "Admin Portal...",
      action: #selector(adminPortalButtonTapped),
      target: self
    )
    return menuItem
  }()
  private lazy var documentationMenuItem: NSMenuItem = {
    let menuItem = createMenuItem(
      menu,
      title: "Documentation...",
      action: #selector(documentationButtonTapped),
      target: self
    )
    return menuItem
  }()
  private lazy var supportMenuItem = createMenuItem(
    menu,
    title: "Support...",
    action: #selector(supportButtonTapped),
    target: self
  )
  private lazy var helpMenuItem: NSMenuItem = {
    let menuItem = NSMenuItem(title: "Help", action: nil, keyEquivalent: "")
    let subMenu = NSMenu()
    subMenu.addItem(documentationMenuItem)
    subMenu.addItem(supportMenuItem)
    menuItem.submenu = subMenu
    return menuItem
  }()

  private lazy var settingsMenuItem = createMenuItem(
    menu,
    title: "Settings",
    action: #selector(settingsButtonTapped),
    key: ",",
    target: self
  )
  private lazy var quitMenuItem: NSMenuItem = {
    let menuItem = createMenuItem(
      menu,
      title: "Quit",
      action: #selector(quitButtonTapped),
      key: "q",
      target: self
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

    if (!model.favorites.ids.isEmpty) {
      menu.addItem(otherResourcesMenuItem)
      menu.addItem(otherResourcesSeparatorMenuItem)
    }

    menu.addItem(aboutMenuItem)
    menu.addItem(adminPortalMenuItem)
    menu.addItem(helpMenuItem)
    menu.addItem(settingsMenuItem)
    menu.addItem(NSMenuItem.separator())
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

  @objc private func signInButtonTapped() {
    NSApp.activate(ignoringOtherApps: true)
    Task { await WebAuthSession.signIn(store: model.store) }
  }

  @objc private func signOutButtonTapped() {
    Task {
      try await model.store.signOut()
    }
  }

  @objc private func settingsButtonTapped() {
    AppViewModel.WindowDefinition.settings.openWindow()
  }

  @objc private func adminPortalButtonTapped() {
    let url = URL(string: model.store.settings.authBaseURL)!
    NSWorkspace.shared.open(url)
  }

  @objc private func documentationButtonTapped() {
    let url = URL(string: "https://www.firezone.dev/kb?utm_source=macos-client")!
    NSWorkspace.shared.open(url)
  }

  @objc private func supportButtonTapped() {
    let url = URL(string: "https://www.firezone.dev/support?utm_source=macos-client")!
    NSWorkspace.shared.open(url)
  }

  @objc private func aboutButtonTapped() {
    NSApp.activate(ignoringOtherApps: true)
    NSApp.orderFrontStandardAboutPanel(self)
  }

  @objc private func quitButtonTapped() {
    Task {
      model.store.stop()
      NSApp.terminate(self)
    }
  }

  private func updateStatusItemIcon(status: NEVPNStatus) {
    statusItem.button?.image = {
      switch status {
      case .invalid, .disconnected:
        self.stopConnectingAnimation()
        return self.signedOutIcon
      case .connected:
        self.stopConnectingAnimation()
        return self.signedInConnectedIcon
      case .connecting, .disconnecting, .reasserting:
        self.startConnectingAnimation()
        return self.connectingAnimationImages.last!
      @unknown default:
        return nil
      }
    }()
  }

  private func startConnectingAnimation() {
    guard connectingAnimationTimer == nil else { return }
    let timer = Timer(timeInterval: 0.25, repeats: true) { [weak self] _ in
      guard let self = self else { return }
      Task {
        await self.connectingAnimationShowNextFrame()
      }
    }
    RunLoop.main.add(timer, forMode: .common)
    connectingAnimationTimer = timer
  }

  private func stopConnectingAnimation() {
    connectingAnimationTimer?.invalidate()
    connectingAnimationTimer = nil
  }

  private func connectingAnimationShowNextFrame() {
    statusItem.button?.image =
    connectingAnimationImages[connectingAnimationImageIndex]
    connectingAnimationImageIndex =
    (connectingAnimationImageIndex + 1) % connectingAnimationImages.count
  }

  private func handleTunnelStatusOrResourcesChanged() {
    let resources = model.resources
    let status = model.status
    // Update "Sign In" / "Sign Out" menu items
    switch status {
    case .invalid:
      signInMenuItem.title = "Requires VPN permission"
      signInMenuItem.target = nil
      signOutMenuItem.isHidden = true
      settingsMenuItem.target = nil
    case .disconnected:
      signInMenuItem.title = "Sign In"
      signInMenuItem.target = self
      signInMenuItem.isEnabled = true
      signOutMenuItem.isHidden = true
      settingsMenuItem.target = self
    case .disconnecting:
      signInMenuItem.title = "Signing out..."
      signInMenuItem.target = self
      signInMenuItem.isEnabled = false
      signOutMenuItem.isHidden = true
      settingsMenuItem.target = self
    case .connected, .reasserting, .connecting:
      let title = "Signed in as \(model.store.actorName ?? "Unknown User")"
      signInMenuItem.title = title
      signInMenuItem.target = nil
      signOutMenuItem.isHidden = false
      settingsMenuItem.target = self
    @unknown default:
      break
    }
    // Update resources "header" menu items
    switch status {
    case .connecting:
      resourcesTitleMenuItem.isHidden = true
      resourcesUnavailableMenuItem.isHidden = false
      resourcesUnavailableReasonMenuItem.isHidden = false
      resourcesUnavailableReasonMenuItem.target = nil
      resourcesUnavailableReasonMenuItem.title = "Connecting…"
      resourcesSeparatorMenuItem.isHidden = false
    case .connected:
      resourcesTitleMenuItem.isHidden = false
      resourcesUnavailableMenuItem.isHidden = true
      resourcesUnavailableReasonMenuItem.isHidden = true
      resourcesTitleMenuItem.title = resourceMenuTitle(resources)
      resourcesSeparatorMenuItem.isHidden = false
    case .reasserting:
      resourcesTitleMenuItem.isHidden = true
      resourcesUnavailableMenuItem.isHidden = false
      resourcesUnavailableReasonMenuItem.isHidden = false
      resourcesUnavailableReasonMenuItem.target = nil
      resourcesUnavailableReasonMenuItem.title = "No network connectivity"
      resourcesSeparatorMenuItem.isHidden = false
    case .disconnecting:
      resourcesTitleMenuItem.isHidden = true
      resourcesUnavailableMenuItem.isHidden = false
      resourcesUnavailableReasonMenuItem.isHidden = false
      resourcesUnavailableReasonMenuItem.target = nil
      resourcesUnavailableReasonMenuItem.title = "Disconnecting…"
      resourcesSeparatorMenuItem.isHidden = false
    case .disconnected, .invalid:
      // We should never be in a state where the tunnel is
      // down but the user is signed in, but we have
      // code to handle it just for the sake of completion.
      resourcesTitleMenuItem.isHidden = true
      resourcesUnavailableMenuItem.isHidden = true
      resourcesUnavailableReasonMenuItem.isHidden = true
      resourcesUnavailableReasonMenuItem.title = "Disconnected"
      resourcesSeparatorMenuItem.isHidden = true
    @unknown default:
      break
    }
    quitMenuItem.title = {
      switch status {
      case .connected, .connecting:
        return "Disconnect and Quit"
      default:
        return "Quit"
      }
    }()
  }

  private func resourceMenuTitle(_ resources: ResourceList) -> String {
    switch resources {
    case .loading:
      return "Loading Resources..."
    case .loaded(let x):
      if x.isEmpty {
        return "No Resources"
      } else {
        return "Resources"
      }
    }
  }

  private func populateResourceMenus(_ newResources: [Resource]) {
    // If we have no favorites, then everything is a favorite
    let hasAnyFavorites = newResources.contains { model.favorites.contains($0.id) }
    let newFavorites = if (hasAnyFavorites) {
      newResources.filter { model.favorites.contains($0.id) || $0.isInternetResource() }
    } else {
      newResources
    }
    let newOthers: [Resource] = if hasAnyFavorites {
      newResources.filter { !model.favorites.contains($0.id) && !$0.isInternetResource() }
    } else {
      []
    }

    populateFavoriteResourcesMenu(newFavorites)
    populateOtherResourcesMenu(newOthers)
  }

  private func populateFavoriteResourcesMenu(_ newFavorites: [Resource]) {
    // Update the menu in place so everything won't vanish if it's open when it updates
    let diff = (newFavorites).difference(
      from: lastShownFavorites,
      by: { $0 == $1 }
    )
    let index = menu.index(of: resourcesTitleMenuItem) + 1
    for change in diff {
      switch change {
      case .insert(let offset, let element, associatedWith: _):
        let menuItem = createResourceMenuItem(resource: element)
        menu.insertItem(menuItem, at: index + offset)
      case .remove(let offset, element: _, associatedWith: _):
        menu.removeItem(at: index + offset)
      }
    }
    lastShownFavorites = newFavorites
  }

  private func populateOtherResourcesMenu(_ newOthers: [Resource]) {
    if (newOthers.isEmpty) {
      removeItemFromMenu(menu: menu, item: otherResourcesMenuItem)
      removeItemFromMenu(menu: menu, item: otherResourcesSeparatorMenuItem)
    } else {
      let i = menu.index(of: aboutMenuItem)
      addItemToMenu(menu: menu, item: otherResourcesMenuItem, at: i)
      addItemToMenu(menu: menu, item: otherResourcesSeparatorMenuItem, at: i + 1)
    }

    // Update the menu in place so everything won't vanish if it's open when it updates
    let diff = (newOthers).difference(
      from: lastShownOthers,
      by: { $0 == $1 }
    )
    for change in diff {
      switch change {
      case .insert(let offset, let element, associatedWith: _):
        let menuItem = createResourceMenuItem(resource: element)
        otherResourcesMenu.insertItem(menuItem, at: offset)
      case .remove(let offset, element: _, associatedWith: _):
        otherResourcesMenu.removeItem(at: offset)
      }
    }
    lastShownOthers = newOthers
  }

  private func addItemToMenu(menu: NSMenu, item: NSMenuItem, at: Int) {
    // Adding an item that already exists will crash the process, so check for it first.
    let i = menu.index(of: otherResourcesMenuItem)
    if (i != -1) {
      // Item's already in the menu, do nothing
      return
    }
    menu.insertItem(otherResourcesMenuItem, at: at)
  }

  private func removeItemFromMenu(menu: NSMenu, item: NSMenuItem) {
    // Removing an item that doesn't exist will crash the process, so check for it first.
    let i = menu.index(of: item)
    if (i == -1) {
      // Item's already not in the menu, do nothing
      return
    }
    menu.removeItem(item)
  }

  private func internetResourceTitle(resource: Resource) -> String {
    let status = !model.store.internetResourceEnabled() && resource.canBeDisabled ? "[OFF]" : "[ON]"

    return status + " " + resource.name
  }

  private func resourceTitle(resource: Resource) -> String {
    if resource.isInternetResource() {
      return internetResourceTitle(resource: resource)
    }

    return resource.name
  }

  private func createResourceMenuItem(resource: Resource) -> NSMenuItem {
    let item = NSMenuItem(title: resourceTitle(resource: resource), action: nil, keyEquivalent: "")

    item.isHidden = false
    item.submenu = createSubMenu(resource: resource)

    return item
  }

  private func internetResourceToggleTitle() -> String {
    model.isInternetResourceEnabled() ? "Disable this resource" : "Enable this resource"
  }

  private func nonInternetResourceHeader(resource: Resource) -> NSMenu {
    let subMenu = NSMenu()

    // AddressDescription first -- will be most common action
    let resourceAddressDescriptionItem = NSMenuItem()
    if let addressDescription = resource.addressDescription {
      resourceAddressDescriptionItem.title = addressDescription

      if let url = URL(string: addressDescription),
         let _ = url.host {
        // Looks like a URL, so allow opening it
        resourceAddressDescriptionItem.action = #selector(resourceURLTapped(_:))
        resourceAddressDescriptionItem.toolTip = "Click to open"

        // TODO: Expose markdown support? Blocked by Tauri clients.
        // Using Markdown here only to highlight the URL
        resourceAddressDescriptionItem.attributedTitle = try? NSAttributedString(markdown: "**[\(addressDescription)](\(addressDescription))**")
      } else {
        resourceAddressDescriptionItem.attributedTitle = try? NSAttributedString(markdown: "**\(addressDescription)**")
        resourceAddressDescriptionItem.action = #selector(resourceValueTapped(_:))
        resourceAddressDescriptionItem.toolTip = "Click to copy"
      }
    } else {
      // Show Address first if addressDescription is missing
      resourceAddressDescriptionItem.title = resource.address! // Address is none only for non-internet resource
      resourceAddressDescriptionItem.action = #selector(resourceValueTapped(_:))
    }
    resourceAddressDescriptionItem.isEnabled = true
    resourceAddressDescriptionItem.target = self
    subMenu.addItem(resourceAddressDescriptionItem)

    subMenu.addItem(NSMenuItem.separator())

    let resourceSectionItem = NSMenuItem()
    resourceSectionItem.title = "Resource"
    resourceSectionItem.isEnabled = false
    subMenu.addItem(resourceSectionItem)

    // Resource name
    let resourceNameItem = NSMenuItem()
    resourceNameItem.action = #selector(resourceValueTapped(_:))
    resourceNameItem.title = resource.name
    resourceNameItem.toolTip = "Resource name (click to copy)"
    resourceNameItem.isEnabled = true
    resourceNameItem.target = self
    subMenu.addItem(resourceNameItem)

    // Resource address
    let resourceAddressItem = NSMenuItem()
    resourceAddressItem.action = #selector(resourceValueTapped(_:))
    resourceAddressItem.title = resource.address!
    resourceAddressItem.toolTip = "Resource address (click to copy)"
    resourceAddressItem.isEnabled = true
    resourceAddressItem.target = self
    subMenu.addItem(resourceAddressItem)

    let toggleFavoriteItem = NSMenuItem()

    if model.favorites.contains(resource.id) {
      toggleFavoriteItem.action = #selector(removeFavoriteTapped(_:))
      toggleFavoriteItem.title = "Remove from favorites"
      toggleFavoriteItem.toolTip = "Click to remove this Resource from Favorites"
    } else {
      toggleFavoriteItem.action = #selector(addFavoriteTapped(_:))
      toggleFavoriteItem.title = "Add to favorites"
      toggleFavoriteItem.toolTip = "Click to add this Resource to Favorites"
    }
    toggleFavoriteItem.isEnabled = true
    toggleFavoriteItem.representedObject = resource.id
    toggleFavoriteItem.target = self
    subMenu.addItem(toggleFavoriteItem)

    return subMenu
  }

  private func internetResourceHeader(resource: Resource) -> NSMenu {
    let subMenu = NSMenu()
    let description = NSMenuItem()

    description.title = "All network traffic"
    description.isEnabled = false

    subMenu.addItem(description)

    // Resource enable / disable toggle
    if resource.canBeDisabled {
      subMenu.addItem(NSMenuItem.separator())
      let enableToggle = NSMenuItem()
      enableToggle.action = #selector(internetResourceToggle(_:))
      enableToggle.title = internetResourceToggleTitle()
      enableToggle.toolTip = "Enable or disable resource"
      enableToggle.isEnabled = true
      enableToggle.target = self
      enableToggle.representedObject = resource.id
      subMenu.addItem(enableToggle)
    }

    return subMenu
  }

  private func resourceHeader(resource: Resource) -> NSMenu {
    if resource.isInternetResource() {
      internetResourceHeader(resource: resource)
    } else {
      nonInternetResourceHeader(resource: resource)
    }
  }

  private func createSubMenu(resource: Resource) -> NSMenu {
    let siteSectionItem = NSMenuItem()
    let siteNameItem = NSMenuItem()
    let siteStatusItem = NSMenuItem()

    let subMenu = resourceHeader(resource: resource)

    // Site details
    if let site = resource.sites.first {
      subMenu.addItem(NSMenuItem.separator())

      siteSectionItem.title = "Site"
      siteSectionItem.isEnabled = false
      subMenu.addItem(siteSectionItem)

      // Site name
      siteNameItem.title = site.name
      siteNameItem.action = #selector(resourceValueTapped(_:))
      siteNameItem.toolTip = "Site name (click to copy)"
      siteNameItem.isEnabled = true
      siteNameItem.target = self
      subMenu.addItem(siteNameItem)

      // Site status
      siteStatusItem.action = #selector(resourceValueTapped(_:))
      siteStatusItem.title = resource.status.toSiteStatus()
      siteStatusItem.toolTip = "\(resource.status.toSiteStatusTooltip()) (click to copy)"
      siteStatusItem.state = statusToState(status: resource.status)
      siteStatusItem.isEnabled = true
      siteStatusItem.target = self
      if let onImage = NSImage(named: NSImage.statusAvailableName),
         let offImage = NSImage(named: NSImage.statusUnavailableName),
         let mixedImage = NSImage(named: NSImage.statusNoneName) {
        siteStatusItem.onStateImage = onImage
        siteStatusItem.offStateImage = offImage
        siteStatusItem.mixedStateImage = mixedImage
      }
      subMenu.addItem(siteStatusItem)
    }

    return subMenu
  }

  @objc private func resourceValueTapped(_ sender: AnyObject?) {
    if let value = (sender as? NSMenuItem)?.title {
      copyToClipboard(value)
    }
  }

  @objc private func internetResourceToggle(_ sender: NSMenuItem) {
    let id = sender.representedObject as! String

    self.model.store.toggleInternetResource(enabled: !model.store.internetResourceEnabled())
    sender.title = internetResourceToggleTitle()
  }

  @objc private func resourceURLTapped(_ sender: AnyObject?) {
    if let value = (sender as? NSMenuItem)?.title {
      // URL has already been validated
      NSWorkspace.shared.open(URL(string: value)!)
    }
  }

  @objc private func addFavoriteTapped(_ sender: NSMenuItem) {
    let id = sender.representedObject as! String
    setFavorited(id: id, favorited: true)
  }

  @objc private func removeFavoriteTapped(_ sender: NSMenuItem) {
    let id = sender.representedObject as! String
    setFavorited(id: id, favorited: false)
  }

  private func setFavorited(id: String, favorited: Bool) {
    if favorited {
      model.favorites.add(id)
    } else {
      model.favorites.remove(id)
    }
  }

  private func copyToClipboard(_ string: String) {
    let pasteBoard = NSPasteboard.general
    pasteBoard.clearContents()
    pasteBoard.writeObjects([string as NSString])
  }

  private func statusToState(status: ResourceStatus) -> NSControl.StateValue {
    switch status {
    case .offline:
      return .off
    case .online:
      return .on
    case .unknown:
      return .mixed
    }
  }
}

extension MenuBar: NSMenuDelegate {
}

extension NSImage {
  func resized(to newSize: NSSize) -> NSImage {
    let newImage = NSImage(size: newSize)
    newImage.lockFocus()
    self.draw(in: NSRect(origin: .zero, size: newSize), from: NSRect(origin: .zero, size: self.size), operation: .copy, fraction: 1.0)
    newImage.unlockFocus()
    newImage.size = newSize
    return newImage
  }
}
#endif
