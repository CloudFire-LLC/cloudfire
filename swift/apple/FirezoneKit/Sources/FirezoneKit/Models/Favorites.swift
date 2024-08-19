import Foundation

public class Favorites: ObservableObject {
  private static let key = "favoriteResourceIDs"
  @Published private(set) var ids: Set<String>

  public init() {
    ids = Favorites.load()
  }

  func contains(_ id: String) -> Bool {
    return ids.contains(id)
  }

  func reset() {
    ids = Set()
    save()
  }

  func add(_ id: String) {
    ids.insert(id)
    save()
  }

  func remove(_ id: String) {
    ids.remove(id)
    save()
  }

  private func save() {
    // It's a run-time exception if we pass the `Set` directly here
    let ids = Array(ids)
    UserDefaults.standard.set(ids, forKey: Favorites.key)
    // Trigger reactive updates
    self.ids = self.ids
  }

  private static func load() -> Set<String> {
    if let ids = UserDefaults.standard.stringArray(forKey: key) {
      return Set(ids)
    }
    return []
  }
}
