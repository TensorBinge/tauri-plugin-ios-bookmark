import Foundation

final class BookmarkStore {
  private let defaults: UserDefaults
  private let prefix = "tauri.ios_bookmark."

  init(defaults: UserDefaults = .standard) {
    self.defaults = defaults
  }

  func save(bookmarkData: Data, fileName: String) -> String {
    let id = UUID().uuidString
    defaults.set(bookmarkData, forKey: prefix + "data." + id)
    defaults.set(fileName, forKey: prefix + "name." + id)
    return id
  }

  func getBookmarkData(id: String) -> Data? {
    defaults.data(forKey: prefix + "data." + id)
  }

  func getFileName(id: String) -> String? {
    defaults.string(forKey: prefix + "name." + id)
  }

  func update(id: String, bookmarkData: Data, fileName: String?) {
    defaults.set(bookmarkData, forKey: prefix + "data." + id)
    if let fileName {
      defaults.set(fileName, forKey: prefix + "name." + id)
    }
  }

  func remove(id: String) {
    defaults.removeObject(forKey: prefix + "data." + id)
    defaults.removeObject(forKey: prefix + "name." + id)
  }
}
