import XCTest
@testable import tauri_plugin_ios_bookmark

final class ScopedFolderPathResolverTests: XCTestCase {
  func testResolveScopedChildUrlKeepsScopedRootUrlWhenTargetMatchesRootAlias() throws {
    let folderUrl = URL(fileURLWithPath: "/private/var/mobile/Library/Mobile Documents/com~apple~CloudDocs/MarkScope", isDirectory: true)
    let resolved = try resolveScopedChildUrlInBookmarkScope(
      targetPath: "/var/mobile/Library/Mobile Documents/com~apple~CloudDocs/MarkScope",
      folderUrl: folderUrl
    )

    XCTAssertEqual(resolved.path, folderUrl.path)
  }

  func testResolveScopedChildUrlBuildsDescendantsFromScopedFolderUrl() throws {
    let folderUrl = URL(fileURLWithPath: "/private/var/mobile/Library/Mobile Documents/com~apple~CloudDocs/MarkScope", isDirectory: true)
    let resolved = try resolveScopedChildUrlInBookmarkScope(
      targetPath: "/var/mobile/Library/Mobile Documents/com~apple~CloudDocs/MarkScope/Drafts/chapter.md",
      folderUrl: folderUrl
    )

    XCTAssertEqual(
      resolved.path,
      folderUrl
        .appendingPathComponent("Drafts", isDirectory: true)
        .appendingPathComponent("chapter.md", isDirectory: false)
        .path
    )
  }
}