#if os(iOS)
import Foundation
#if os(iOS)
import Tauri
#endif
import UniformTypeIdentifiers

// MARK: - Argument DTOs

private struct PickFileBookmarkArgs: Decodable {
  let request: PickFileBookmarkRequestDTO?
}

private struct PickFolderBookmarkArgs: Decodable {
  let request: PickFolderBookmarkRequestDTO?
}

private struct PickFileBookmarkRequestDTO: Decodable {
  let suggestedName: String?
  let skipContent: Bool?
}

private struct PickFolderBookmarkRequestDTO: Decodable {
  let suggestedName: String?
  let requireEmpty: Bool?
}

// MARK: - Plugin

final class BookmarkPlugin: Plugin, UIDocumentPickerDelegate, UIAdaptivePresentationControllerDelegate {
  private var pendingInvoke: Invoke?
  private var pendingPickRequest: PickFileBookmarkRequestDTO?
  private var pendingFolderPickRequest: PickFolderBookmarkRequestDTO?
  private var pendingPickerKind: PendingPickerKind?
  private let store = BookmarkStore()

  private enum PendingPickerKind {
    case file
    case folder
  }

  // ── File bookmark operations ──────────────────────────────────────

  @objc public func pickFileBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: pickFileBookmark entered mainThread=\(Thread.isMainThread)", category: "ios-bookmark")

    let args = (try? invoke.parseArgs(PickFileBookmarkArgs.self)) ?? PickFileBookmarkArgs(request: nil)

    let presentPicker = {
      Logger.info("[ios-bookmark] swift plugin: pickFileBookmark on main thread", category: "ios-bookmark")
      guard let presenter = self.manager.viewController else {
        Logger.error("[ios-bookmark] swift plugin: no active view controller", category: "ios-bookmark")
        invoke.reject("\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.nativeError.rawValue):No active view controller")
        return
      }

      guard self.pendingInvoke == nil else {
        Logger.error("[ios-bookmark] swift plugin: picker already in progress", category: "ios-bookmark")
        invoke.reject("\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.nativeError.rawValue):Picker already in progress")
        return
      }

      let picker = self.makeFilePicker()
      self.pendingInvoke = invoke
      self.pendingPickRequest = args.request
      self.pendingPickerKind = .file

      if let directoryUrl = self.initialDirectoryUrl(forSuggestedName: args.request?.suggestedName) {
        picker.directoryURL = directoryUrl
      }

      Logger.info("[ios-bookmark] swift plugin: presenting UIDocumentPickerViewController", category: "ios-bookmark")
      presenter.present(picker, animated: true, completion: nil)
    }

    if Thread.isMainThread {
      presentPicker()
    } else {
      DispatchQueue.main.async(execute: presentPicker)
    }
  }

  @objc public func readFileBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: readFileBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      guard let bookmarkData = store.getBookmarkData(id: args.id) else {
        throw bookmarkError(.notFound, "No bookmark found for id: \(args.id)")
      }

      var stale = false
      let url = try resolveSecurityScopedBookmarkUrl(bookmarkData, stale: &stale)

      guard url.startAccessingSecurityScopedResource() else {
        throw bookmarkError(.permissionDenied, "Failed to access security-scoped resource for id: \(args.id)")
      }
      defer { url.stopAccessingSecurityScopedResource() }

      if stale {
        let refreshedBookmark = try makeSecurityScopedBookmarkData(for: url)
        let refreshedName = try? url.resourceValues(forKeys: [.nameKey]).name
        store.update(id: args.id, bookmarkData: refreshedBookmark, fileName: refreshedName)
      }

      let content = try coordinatedRead(url: url)
      let fileName = store.getFileName(id: args.id) ?? url.lastPathComponent
      Logger.info("[ios-bookmark] swift plugin: readFileBookmark resolved for \(fileName)", category: "ios-bookmark")
      invoke.resolve(ReadResultDTO(fileName: fileName, filePath: url.path, content: content))
    } catch {
      Logger.error("[ios-bookmark] swift plugin: readFileBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func readFileBookmarkData(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: readFileBookmarkData entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      guard let bookmarkData = store.getBookmarkData(id: args.id) else {
        throw bookmarkError(.notFound, "No bookmark found for id: \(args.id)")
      }

      var stale = false
      let url = try resolveSecurityScopedBookmarkUrl(bookmarkData, stale: &stale)

      guard url.startAccessingSecurityScopedResource() else {
        throw bookmarkError(.permissionDenied, "Failed to access security-scoped resource for id: \(args.id)")
      }
      defer { url.stopAccessingSecurityScopedResource() }

      if stale {
        let refreshedBookmark = try makeSecurityScopedBookmarkData(for: url)
        let refreshedName = try? url.resourceValues(forKeys: [.nameKey]).name
        store.update(id: args.id, bookmarkData: refreshedBookmark, fileName: refreshedName)
      }

      let binaryData = try coordinatedReadData(url: url)
      let mimeType = mimeTypeForFile(at: url)
      let fileName = store.getFileName(id: args.id) ?? url.lastPathComponent
      Logger.info("[ios-bookmark] swift plugin: readFileBookmarkData resolved for \(fileName)", category: "ios-bookmark")
      invoke.resolve(
        DataResultDTO(
          name: fileName,
          path: url.path,
          mimeType: mimeType,
          base64Data: binaryData.base64EncodedString()
        )
      )
    } catch {
      Logger.error("[ios-bookmark] swift plugin: readFileBookmarkData error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func writeFileBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: writeFileBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let content: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      guard let bookmarkData = store.getBookmarkData(id: args.id) else {
        throw bookmarkError(.notFound, "No bookmark found for id: \(args.id)")
      }

      var stale = false
      let url = try resolveSecurityScopedBookmarkUrl(bookmarkData, stale: &stale)

      guard url.startAccessingSecurityScopedResource() else {
        throw bookmarkError(.permissionDenied, "Failed to access security-scoped resource for id: \(args.id)")
      }
      defer { url.stopAccessingSecurityScopedResource() }

      if stale {
        let refreshedBookmark = try makeSecurityScopedBookmarkData(for: url)
        let refreshedName = try? url.resourceValues(forKeys: [.nameKey]).name
        store.update(id: args.id, bookmarkData: refreshedBookmark, fileName: refreshedName)
      }

      try coordinatedWrite(url: url, content: args.content)
      Logger.info("[ios-bookmark] swift plugin: writeFileBookmark resolved for \(url.lastPathComponent)", category: "ios-bookmark")
      invoke.resolve()
    } catch {
      Logger.error("[ios-bookmark] swift plugin: writeFileBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func writeFileBookmarkData(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: writeFileBookmarkData entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let data: [UInt8]
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      guard let bookmarkData = store.getBookmarkData(id: args.id) else {
        throw bookmarkError(.notFound, "No bookmark found for id: \(args.id)")
      }

      var stale = false
      let url = try resolveSecurityScopedBookmarkUrl(bookmarkData, stale: &stale)

      guard url.startAccessingSecurityScopedResource() else {
        throw bookmarkError(.permissionDenied, "Failed to access security-scoped resource for id: \(args.id)")
      }
      defer { url.stopAccessingSecurityScopedResource() }

      if stale {
        let refreshedBookmark = try makeSecurityScopedBookmarkData(for: url)
        let refreshedName = try? url.resourceValues(forKeys: [.nameKey]).name
        store.update(id: args.id, bookmarkData: refreshedBookmark, fileName: refreshedName)
      }

      try coordinatedWriteData(url: url, data: Data(args.data))
      Logger.info("[ios-bookmark] swift plugin: writeFileBookmarkData resolved for \(url.lastPathComponent)", category: "ios-bookmark")
      invoke.resolve()
    } catch {
      Logger.error("[ios-bookmark] swift plugin: writeFileBookmarkData error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  // ── Folder bookmark operations ────────────────────────────────────

  @objc public func pickFolderBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: pickFolderBookmark entered mainThread=\(Thread.isMainThread)", category: "ios-bookmark")

    let args = (try? invoke.parseArgs(PickFolderBookmarkArgs.self)) ?? PickFolderBookmarkArgs(request: nil)

    let presentPicker = {
      Logger.info("[ios-bookmark] swift plugin: pickFolderBookmark on main thread", category: "ios-bookmark")
      guard let presenter = self.manager.viewController else {
        Logger.error("[ios-bookmark] swift plugin: no active view controller", category: "ios-bookmark")
        invoke.reject("\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.nativeError.rawValue):No active view controller")
        return
      }

      guard self.pendingInvoke == nil else {
        Logger.error("[ios-bookmark] swift plugin: picker already in progress", category: "ios-bookmark")
        invoke.reject("\(bookmarkNativeErrorPrefix):\(BookmarkErrorCode.nativeError.rawValue):Picker already in progress")
        return
      }

      let picker = self.makeFolderPicker()
      self.pendingInvoke = invoke
      self.pendingFolderPickRequest = args.request
      self.pendingPickerKind = .folder

      if let directoryUrl = self.initialDirectoryUrl(forSuggestedNameFolder: args.request?.suggestedName) {
        picker.directoryURL = directoryUrl
      }

      Logger.info("[ios-bookmark] swift plugin: presenting folder UIDocumentPickerViewController", category: "ios-bookmark")
      presenter.present(picker, animated: true, completion: nil)
    }

    if Thread.isMainThread {
      presentPicker()
    } else {
      DispatchQueue.main.async(execute: presentPicker)
    }
  }

  @objc public func listFolderBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: listFolderBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let path: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      let folderUrl = try resolveScopedFolderUrl(id: args.id)
      defer { folderUrl.stopAccessingSecurityScopedResource() }
      let targetUrl = try resolveScopedChildUrl(targetPath: args.path, folderUrl: folderUrl)
      let entries = try coordinatedList(url: targetUrl)
      invoke.resolve(entries)
    } catch {
      Logger.error("[ios-bookmark] swift plugin: listFolderBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func readFolderBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: readFolderBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let path: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      guard let bookmarkData = store.getBookmarkData(id: args.id) else {
        throw bookmarkError(.notFound, "No folder bookmark found for id: \(args.id)")
      }

      var stale = false
      let folderUrl = try resolveSecurityScopedBookmarkUrl(bookmarkData, stale: &stale)

      guard urlIsWithinFolder(targetPath: args.path, folderPath: folderUrl.path) else {
        throw bookmarkError(.permissionDenied, "Requested path is outside the bookmarked folder")
      }

      guard folderUrl.startAccessingSecurityScopedResource() else {
        throw bookmarkError(.permissionDenied, "Failed to access security-scoped folder for id: \(args.id)")
      }
      defer { folderUrl.stopAccessingSecurityScopedResource() }

      if stale {
        let refreshedBookmark = try makeSecurityScopedBookmarkData(for: folderUrl)
        let refreshedName = try? folderUrl.resourceValues(forKeys: [.nameKey]).name
        store.updateFolder(id: args.id, bookmarkData: refreshedBookmark, folderName: refreshedName)
      }

      let targetUrl = try resolveScopedChildUrl(targetPath: args.path, folderUrl: folderUrl)
      let content = try coordinatedRead(url: targetUrl)
      Logger.info("[ios-bookmark] swift plugin: readFolderBookmark resolved for \(targetUrl.lastPathComponent)", category: "ios-bookmark")
      invoke.resolve(ReadResultDTO(fileName: targetUrl.lastPathComponent, filePath: targetUrl.path, content: content))
    } catch {
      Logger.error("[ios-bookmark] swift plugin: readFolderBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func readFolderBookmarkData(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: readFolderBookmarkData entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let path: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      guard let bookmarkData = store.getBookmarkData(id: args.id) else {
        throw bookmarkError(.notFound, "No folder bookmark found for id: \(args.id)")
      }

      var stale = false
      let folderUrl = try resolveSecurityScopedBookmarkUrl(bookmarkData, stale: &stale)

      guard urlIsWithinFolder(targetPath: args.path, folderPath: folderUrl.path) else {
        throw bookmarkError(.permissionDenied, "Requested path is outside the bookmarked folder")
      }

      guard folderUrl.startAccessingSecurityScopedResource() else {
        throw bookmarkError(.permissionDenied, "Failed to access security-scoped folder for id: \(args.id)")
      }
      defer { folderUrl.stopAccessingSecurityScopedResource() }

      if stale {
        let refreshedBookmark = try makeSecurityScopedBookmarkData(for: folderUrl)
        let refreshedName = try? folderUrl.resourceValues(forKeys: [.nameKey]).name
        store.updateFolder(id: args.id, bookmarkData: refreshedBookmark, folderName: refreshedName)
      }

      let targetUrl = try resolveScopedChildUrl(targetPath: args.path, folderUrl: folderUrl)
      let binaryData = try coordinatedReadData(url: targetUrl)
      let mimeType = mimeTypeForFile(at: targetUrl)

      Logger.info("[ios-bookmark] swift plugin: readFolderBookmarkData resolved for \(targetUrl.lastPathComponent)", category: "ios-bookmark")
      invoke.resolve(
        DataResultDTO(
          name: targetUrl.lastPathComponent,
          path: targetUrl.path,
          mimeType: mimeType,
          base64Data: binaryData.base64EncodedString()
        )
      )
    } catch {
      Logger.error("[ios-bookmark] swift plugin: readFolderBookmarkData error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func writeFolderBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: writeFolderBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let path: String
      let content: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      guard let bookmarkData = store.getBookmarkData(id: args.id) else {
        throw bookmarkError(.notFound, "No folder bookmark found for id: \(args.id)")
      }

      var stale = false
      let folderUrl = try resolveSecurityScopedBookmarkUrl(bookmarkData, stale: &stale)

      guard urlIsWithinFolder(targetPath: args.path, folderPath: folderUrl.path) else {
        throw bookmarkError(.permissionDenied, "Requested path is outside the bookmarked folder")
      }

      guard folderUrl.startAccessingSecurityScopedResource() else {
        throw bookmarkError(.permissionDenied, "Failed to access security-scoped folder for id: \(args.id)")
      }
      defer { folderUrl.stopAccessingSecurityScopedResource() }

      if stale {
        let refreshedBookmark = try makeSecurityScopedBookmarkData(for: folderUrl)
        let refreshedName = try? folderUrl.resourceValues(forKeys: [.nameKey]).name
        store.updateFolder(id: args.id, bookmarkData: refreshedBookmark, folderName: refreshedName)
      }

      let targetUrl = try resolveScopedChildUrl(targetPath: args.path, folderUrl: folderUrl)
      try coordinatedWrite(url: targetUrl, content: args.content)
      Logger.info("[ios-bookmark] swift plugin: writeFolderBookmark resolved for \(targetUrl.lastPathComponent)", category: "ios-bookmark")
      invoke.resolve()
    } catch {
      Logger.error("[ios-bookmark] swift plugin: writeFolderBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func writeFolderBookmarkData(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: writeFolderBookmarkData entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let path: String
      let data: [UInt8]
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      guard let bookmarkData = store.getBookmarkData(id: args.id) else {
        throw bookmarkError(.notFound, "No folder bookmark found for id: \(args.id)")
      }

      var stale = false
      let folderUrl = try resolveSecurityScopedBookmarkUrl(bookmarkData, stale: &stale)

      guard urlIsWithinFolder(targetPath: args.path, folderPath: folderUrl.path) else {
        throw bookmarkError(.permissionDenied, "Requested path is outside the bookmarked folder")
      }

      guard folderUrl.startAccessingSecurityScopedResource() else {
        throw bookmarkError(.permissionDenied, "Failed to access security-scoped folder for id: \(args.id)")
      }
      defer { folderUrl.stopAccessingSecurityScopedResource() }

      if stale {
        let refreshedBookmark = try makeSecurityScopedBookmarkData(for: folderUrl)
        let refreshedName = try? folderUrl.resourceValues(forKeys: [.nameKey]).name
        store.updateFolder(id: args.id, bookmarkData: refreshedBookmark, folderName: refreshedName)
      }

      let targetUrl = try resolveScopedChildUrl(targetPath: args.path, folderUrl: folderUrl)
      try coordinatedWriteData(url: targetUrl, data: Data(args.data))
      Logger.info("[ios-bookmark] swift plugin: writeFolderBookmarkData resolved for \(targetUrl.lastPathComponent)", category: "ios-bookmark")
      invoke.resolve()
    } catch {
      Logger.error("[ios-bookmark] swift plugin: writeFolderBookmarkData error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  // ── Folder-scoped mutations ───────────────────────────────────────

  @objc public func createDir(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: createDir entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let parentPath: String
      let name: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      let folderUrl = try resolveScopedFolderUrl(id: args.id)
      defer { folderUrl.stopAccessingSecurityScopedResource() }
      let parentUrl = try resolveScopedChildUrl(targetPath: args.parentPath, folderUrl: folderUrl)
      let name = try sanitizeWorkspaceComponent(args.name)
      let entry = try coordinatedCreateFolder(parentUrl: parentUrl, name: name)
      invoke.resolve(entry)
    } catch {
      Logger.error("[ios-bookmark] swift plugin: createDir error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func createFile(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: createFile entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let parentPath: String
      let name: String
      let content: String?
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      let folderUrl = try resolveScopedFolderUrl(id: args.id)
      defer { folderUrl.stopAccessingSecurityScopedResource() }
      let parentUrl = try resolveScopedChildUrl(targetPath: args.parentPath, folderUrl: folderUrl)
      let name = try sanitizeWorkspaceComponent(args.name)
      let entry = try coordinatedCreateFile(parentUrl: parentUrl, name: name, content: args.content ?? "")
      invoke.resolve(entry)
    } catch {
      Logger.error("[ios-bookmark] swift plugin: createFile error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func rename(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: rename entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let path: String
      let newName: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      let folderUrl = try resolveScopedFolderUrl(id: args.id)
      defer { folderUrl.stopAccessingSecurityScopedResource() }
      let targetUrl = try resolveScopedChildUrl(targetPath: args.path, folderUrl: folderUrl)
      let resourceValues = try targetUrl.resourceValues(forKeys: [.isDirectoryKey])
      let sanitizedName = try sanitizeWorkspaceComponent(args.newName)
      let entry = try coordinatedRename(url: targetUrl, name: sanitizedName, isDirectory: resourceValues.isDirectory ?? false)
      invoke.resolve(entry)
    } catch {
      Logger.error("[ios-bookmark] swift plugin: rename error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func move(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: move entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let srcPath: String
      let destParentPath: String
      let name: String?
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      let folderUrl = try resolveScopedFolderUrl(id: args.id)
      defer { folderUrl.stopAccessingSecurityScopedResource() }
      let sourceUrl = try resolveScopedChildUrl(targetPath: args.srcPath, folderUrl: folderUrl)
      let destinationParentUrl = try resolveScopedChildUrl(targetPath: args.destParentPath, folderUrl: folderUrl)
      let sourceValues = try sourceUrl.resourceValues(forKeys: [.isDirectoryKey])
      let isDirectory = sourceValues.isDirectory ?? false
      let moveName = try args.name.map { try sanitizeWorkspaceComponent($0) } ?? sourceUrl.lastPathComponent
      let entry = try coordinatedMove(
        sourceUrl: sourceUrl,
        destinationParentUrl: destinationParentUrl,
        name: moveName,
        isDirectory: isDirectory
      )
      invoke.resolve(entry)
    } catch {
      Logger.error("[ios-bookmark] swift plugin: move error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func remove(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: remove entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let path: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      let folderUrl = try resolveScopedFolderUrl(id: args.id)
      defer { folderUrl.stopAccessingSecurityScopedResource() }
      let targetUrl = try resolveScopedChildUrl(targetPath: args.path, folderUrl: folderUrl)
      try coordinatedDelete(url: targetUrl)
      invoke.resolve()
    } catch {
      Logger.error("[ios-bookmark] swift plugin: remove error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  // ── Lifecycle ─────────────────────────────────────────────────────

  @objc public func checkBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: checkBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      guard let bookmarkData = store.getBookmarkData(id: args.id) else {
        invoke.resolve(false)
        return
      }

      var stale = false
      let url = try? resolveSecurityScopedBookmarkUrl(bookmarkData, stale: &stale)

      guard let resolvedUrl = url else {
        invoke.resolve(false)
        return
      }

      let isAccessible = FileManager.default.fileExists(atPath: resolvedUrl.path)
      invoke.resolve(isAccessible)
    } catch {
      Logger.error("[ios-bookmark] swift plugin: checkBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.resolve(false)
    }
  }

  @objc public func releaseBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: releaseBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      store.remove(id: args.id)
      Logger.info("[ios-bookmark] swift plugin: releaseBookmark resolved", category: "ios-bookmark")
      invoke.resolve()
    } catch {
      Logger.error("[ios-bookmark] swift plugin: releaseBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  // ── UIDocumentPickerDelegate ──────────────────────────────────────

  public func documentPicker(
    _ controller: UIDocumentPickerViewController,
    didPickDocumentsAt urls: [URL]
  ) {
    Logger.info("[ios-bookmark] swift plugin: didPickDocumentsAt count=\(urls.count)", category: "ios-bookmark")
    guard let url = urls.first else {
      Logger.info("[ios-bookmark] swift plugin: didPickDocumentsAt without url", category: "ios-bookmark")
      resolvePendingCancellation(reason: "No file selected")
      return
    }

    switch pendingPickerKind {
    case .folder:
      resolvePickedFolder(url)
    case .file, .none:
      resolvePickedFile(url)
    }
  }

  public func documentPickerWasCancelled(_ controller: UIDocumentPickerViewController) {
    Logger.info("[ios-bookmark] swift plugin: documentPickerWasCancelled", category: "ios-bookmark")
    resolvePendingCancellation(reason: "User cancelled")
  }

  public func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
    Logger.info("[ios-bookmark] swift plugin: presentationControllerDidDismiss", category: "ios-bookmark")
    resolvePendingCancellation(reason: "Picker dismissed")
  }

  // ── Resolve helpers ───────────────────────────────────────────────

  private func resolvePickedFile(_ url: URL) {
    let skipContent = pendingPickRequest?.skipContent ?? false

    guard url.startAccessingSecurityScopedResource() else {
      Logger.error("[ios-bookmark] swift plugin: startAccessingSecurityScopedResource failed for \(url)", category: "ios-bookmark")
      rejectPendingInvoke(bookmarkError(.permissionDenied, "Failed to access security-scoped resource"))
      return
    }
    defer { url.stopAccessingSecurityScopedResource() }

    do {
      let bookmarkData = try makeSecurityScopedBookmarkData(for: url)
      let fileName = url.lastPathComponent
      let content: String
      if skipContent {
        content = ""
      } else {
        content = try coordinatedRead(url: url)
      }
      let mimeType = mimeTypeForFile(at: url)
      let id = store.save(bookmarkData: bookmarkData, fileName: fileName)
      Logger.info("[ios-bookmark] swift plugin: resolving pickFileBookmark for \(fileName)", category: "ios-bookmark")

      pendingInvoke?.resolve(
        FileBookmarkResultDTO(id: id, name: fileName, path: url.path, content: content, mimeType: mimeType)
      )
    } catch {
      Logger.error("[ios-bookmark] swift plugin: pickFileBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      rejectPendingInvoke(error)
      return
    }

    clearPendingPickState()
  }

  private func resolvePickedFolder(_ url: URL) {
    guard url.startAccessingSecurityScopedResource() else {
      Logger.error("[ios-bookmark] swift plugin: startAccessingSecurityScopedResource failed for folder \(url)", category: "ios-bookmark")
      rejectPendingInvoke(bookmarkError(.permissionDenied, "Failed to access security-scoped folder"))
      return
    }
    defer { url.stopAccessingSecurityScopedResource() }

    do {
      if pendingFolderPickRequest?.requireEmpty == true {
        try assertFolderIsEmpty(url: url)
      }

      var bookmarkData = try makeSecurityScopedBookmarkData(for: url)

      // Resolve the bookmark immediately and return its canonical path. Newly created
      // iCloud folders can be selected through a transient provider URL that differs
      // from the bookmark's later resolved location.
      var bookmarkIsStale = false
      let resolvedFolderUrl = try resolveSecurityScopedBookmarkUrl(bookmarkData, stale: &bookmarkIsStale)
        .resolvingSymlinksInPath()

      if bookmarkIsStale {
        bookmarkData = try makeSecurityScopedBookmarkData(for: resolvedFolderUrl)
      }

      let folderName = (try? resolvedFolderUrl.resourceValues(forKeys: [.nameKey]).name)
        ?? (resolvedFolderUrl.lastPathComponent.isEmpty ? resolvedFolderUrl.path : resolvedFolderUrl.lastPathComponent)
      let id = store.saveFolder(bookmarkData: bookmarkData, folderName: folderName)
      Logger.info("[ios-bookmark] swift plugin: resolving pickFolderBookmark for \(folderName)", category: "ios-bookmark")

      pendingInvoke?.resolve(
        FolderBookmarkResultDTO(id: id, name: folderName, path: resolvedFolderUrl.path)
      )
    } catch {
      Logger.error("[ios-bookmark] swift plugin: pickFolderBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      rejectPendingInvoke(error)
      return
    }

    clearPendingPickState()
  }

  private func resolvePendingCancellation(reason: String) {
    Logger.info("[ios-bookmark] swift plugin: resolving picker cancellation kind=\(String(describing: pendingPickerKind)) reason=\(reason)", category: "ios-bookmark")

    switch pendingPickerKind {
    case .file:
      pendingInvoke?.resolve(Optional<FileBookmarkResultDTO>.none)
    case .folder:
      pendingInvoke?.resolve(Optional<FolderBookmarkResultDTO>.none)
    case .none:
      pendingInvoke?.resolve()
    }

    clearPendingPickState()
  }

  private func clearPendingPickState() {
    pendingInvoke = nil
    pendingPickRequest = nil
    pendingFolderPickRequest = nil
    pendingPickerKind = nil
  }

  // ── File coordination ─────────────────────────────────────────────

  private func coordinatedRead(url: URL) throws -> String {
    Logger.info("[ios-bookmark] swift plugin: coordinatedRead start for \(url.lastPathComponent)", category: "ios-bookmark")
    var content: String?
    var coordinatorError: NSError?
    var readError: Error?
    let coordinator = NSFileCoordinator(filePresenter: nil)

    coordinator.coordinate(readingItemAt: url, options: [], error: &coordinatorError) { readUrl in
      do {
        content = try String(contentsOf: readUrl, encoding: .utf8)
      } catch {
        readError = error
      }
    }

    if let coordinatorError {
      throw coordinatorError
    }
    if let readError {
      throw readError
    }
    guard let content else {
      throw bookmarkError(.ioError, "Failed to read file content")
    }

    Logger.info("[ios-bookmark] swift plugin: coordinatedRead success for \(url.lastPathComponent)", category: "ios-bookmark")
    return content
  }

  private func coordinatedReadData(url: URL) throws -> Data {
    Logger.info("[ios-bookmark] swift plugin: coordinatedReadData start for \(url.lastPathComponent)", category: "ios-bookmark")
    var content: Data?
    var coordinatorError: NSError?
    var readError: Error?
    let coordinator = NSFileCoordinator(filePresenter: nil)

    coordinator.coordinate(readingItemAt: url, options: [], error: &coordinatorError) { readUrl in
      do {
        content = try Data(contentsOf: readUrl)
      } catch {
        readError = error
      }
    }

    if let coordinatorError {
      throw coordinatorError
    }
    if let readError {
      throw readError
    }
    guard let content else {
      throw bookmarkError(.ioError, "Failed to read file data")
    }

    Logger.info("[ios-bookmark] swift plugin: coordinatedReadData success for \(url.lastPathComponent)", category: "ios-bookmark")
    return content
  }

  private func coordinatedWrite(url: URL, content: String) throws {
    Logger.info("[ios-bookmark] swift plugin: coordinatedWrite start for \(url.lastPathComponent)", category: "ios-bookmark")
    var coordinatorError: NSError?
    var writeError: Error?
    let coordinator = NSFileCoordinator(filePresenter: nil)

    coordinator.coordinate(writingItemAt: url, options: .forReplacing, error: &coordinatorError) { writeUrl in
      do {
        try content.write(to: writeUrl, atomically: true, encoding: .utf8)
      } catch {
        writeError = error
      }
    }

    if let coordinatorError {
      throw coordinatorError
    }
    if let writeError {
      throw writeError
    }

    Logger.info("[ios-bookmark] swift plugin: coordinatedWrite success for \(url.lastPathComponent)", category: "ios-bookmark")
  }

  private func coordinatedWriteData(url: URL, data: Data) throws {
    Logger.info("[ios-bookmark] swift plugin: coordinatedWriteData start for \(url.lastPathComponent)", category: "ios-bookmark")
    var coordinatorError: NSError?
    var writeError: Error?
    let coordinator = NSFileCoordinator(filePresenter: nil)

    coordinator.coordinate(writingItemAt: url, options: .forReplacing, error: &coordinatorError) { writeUrl in
      do {
        try data.write(to: writeUrl, options: .atomic)
      } catch {
        writeError = error
      }
    }

    if let coordinatorError {
      throw coordinatorError
    }
    if let writeError {
      throw writeError
    }

    Logger.info("[ios-bookmark] swift plugin: coordinatedWriteData success for \(url.lastPathComponent)", category: "ios-bookmark")
  }

  private func mimeTypeForFile(at url: URL) -> String {
    if #available(iOS 14.0, *) {
      if let contentType = try? url.resourceValues(forKeys: [.contentTypeKey]).contentType,
         let mimeType = contentType.preferredMIMEType {
        return mimeType
      }

      if let inferredType = UTType(filenameExtension: url.pathExtension),
         let mimeType = inferredType.preferredMIMEType {
        return mimeType
      }
    }

    return "application/octet-stream"
  }

  // ── Directory listing ─────────────────────────────────────────────

  private func coordinatedList(url: URL) throws -> [EntryDTO] {
    var entries: [EntryDTO] = []
    var coordinatorError: NSError?
    var listError: Error?
    let coordinator = NSFileCoordinator(filePresenter: nil)

    coordinator.coordinate(readingItemAt: url, options: [], error: &coordinatorError) { readUrl in
      do {
        let resourceKeys: Set<URLResourceKey> = [.isDirectoryKey, .fileSizeKey, .contentModificationDateKey]
        let urls = try FileManager.default.contentsOfDirectory(
          at: readUrl,
          includingPropertiesForKeys: Array(resourceKeys),
          options: [.skipsHiddenFiles]
        )

        entries = try urls.map { entryUrl in
          let values = try entryUrl.resourceValues(forKeys: resourceKeys)
          return EntryDTO(
            name: entryUrl.lastPathComponent,
            path: entryUrl.path,
            isDir: values.isDirectory ?? false,
            size: values.fileSize.map { UInt64($0) },
            mtime: values.contentModificationDate.map { UInt64($0.timeIntervalSince1970 * 1000) }
          )
        }
      } catch {
        listError = error
      }
    }

    if let coordinatorError {
      throw coordinatorError
    }
    if let listError {
      throw listError
    }

    return entries
  }

  // ── Mutation coordination ─────────────────────────────────────────

  private func coordinatedCreateFolder(parentUrl: URL, name: String) throws -> EntryDTO {
    var createdEntry: EntryDTO?
    var coordinatorError: NSError?
    var createError: Error?
    let coordinator = NSFileCoordinator(filePresenter: nil)

    coordinator.coordinate(writingItemAt: parentUrl, options: .forMerging, error: &coordinatorError) { writeUrl in
      do {
        let folderUrl = writeUrl.appendingPathComponent(name, isDirectory: true)
        try FileManager.default.createDirectory(at: folderUrl, withIntermediateDirectories: false, attributes: nil)
        createdEntry = EntryDTO(name: name, path: folderUrl.path, isDir: true, size: 0, mtime: UInt64(Date().timeIntervalSince1970 * 1000))
      } catch {
        createError = error
      }
    }

    if let coordinatorError {
      throw coordinatorError
    }
    if let createError {
      throw createError
    }

    guard let createdEntry else {
      throw bookmarkError(.ioError, "Failed to create folder")
    }

    return createdEntry
  }

  private func coordinatedCreateFile(parentUrl: URL, name: String, content: String) throws -> EntryDTO {
    var createdEntry: EntryDTO?
    var coordinatorError: NSError?
    var createError: Error?
    let coordinator = NSFileCoordinator(filePresenter: nil)

    coordinator.coordinate(writingItemAt: parentUrl, options: .forMerging, error: &coordinatorError) { writeUrl in
      do {
        let fileUrl = writeUrl.appendingPathComponent(name, isDirectory: false)
        try content.write(to: fileUrl, atomically: true, encoding: .utf8)
        createdEntry = EntryDTO(name: name, path: fileUrl.path, isDir: false, size: UInt64(content.lengthOfBytes(using: .utf8)), mtime: UInt64(Date().timeIntervalSince1970 * 1000))
      } catch {
        createError = error
      }
    }

    if let coordinatorError {
      throw coordinatorError
    }
    if let createError {
      throw createError
    }

    guard let createdEntry else {
      throw bookmarkError(.ioError, "Failed to create file")
    }

    return createdEntry
  }

  private func coordinatedRename(url: URL, name: String, isDirectory: Bool) throws -> EntryDTO {
    var renamedEntry: EntryDTO?
    var coordinatorError: NSError?
    var renameError: Error?
    let coordinator = NSFileCoordinator(filePresenter: nil)

    coordinator.coordinate(writingItemAt: url, options: .forMoving, error: &coordinatorError) { movingUrl in
      do {
        let destinationUrl = movingUrl.deletingLastPathComponent().appendingPathComponent(name, isDirectory: isDirectory)

        if movingUrl.path == destinationUrl.path {
          renamedEntry = try makeEntry(url: movingUrl)
          return
        }

        try FileManager.default.moveItem(at: movingUrl, to: destinationUrl)
        renamedEntry = try makeEntry(url: destinationUrl)
      } catch {
        renameError = error
      }
    }

    if let coordinatorError {
      throw coordinatorError
    }
    if let renameError {
      throw renameError
    }

    guard let renamedEntry else {
      throw bookmarkError(.ioError, "Failed to rename item")
    }

    return renamedEntry
  }

  private func coordinatedMove(sourceUrl: URL, destinationParentUrl: URL, name: String, isDirectory: Bool) throws -> EntryDTO {
    var movedEntry: EntryDTO?
    var coordinatorError: NSError?
    var moveError: Error?
    let coordinator = NSFileCoordinator(filePresenter: nil)

    coordinator.coordinate(
      writingItemAt: sourceUrl,
      options: .forMoving,
      writingItemAt: destinationParentUrl,
      options: .forMerging,
      error: &coordinatorError
    ) { movingUrl, destinationParentCoordinatedUrl in
      do {
        let destinationUrl = destinationParentCoordinatedUrl.appendingPathComponent(name, isDirectory: isDirectory)

        if movingUrl.path == destinationUrl.path {
          movedEntry = try makeEntry(url: movingUrl)
          return
        }

        try FileManager.default.moveItem(at: movingUrl, to: destinationUrl)
        movedEntry = try makeEntry(url: destinationUrl)
      } catch {
        moveError = error
      }
    }

    if let coordinatorError {
      throw coordinatorError
    }
    if let moveError {
      throw moveError
    }

    guard let movedEntry else {
      throw bookmarkError(.ioError, "Failed to move item")
    }

    return movedEntry
  }

  private func coordinatedDelete(url: URL) throws {
    var coordinatorError: NSError?
    var deleteError: Error?
    let coordinator = NSFileCoordinator(filePresenter: nil)

    coordinator.coordinate(writingItemAt: url, options: .forDeleting, error: &coordinatorError) { deleteUrl in
      do {
        try FileManager.default.removeItem(at: deleteUrl)
      } catch {
        deleteError = error
      }
    }

    if let coordinatorError {
      throw coordinatorError
    }
    if let deleteError {
      throw deleteError
    }
  }

  private func makeEntry(url: URL) throws -> EntryDTO {
    let resourceKeys: Set<URLResourceKey> = [.isDirectoryKey, .fileSizeKey, .contentModificationDateKey]
    let values = try url.resourceValues(forKeys: resourceKeys)

    return EntryDTO(
      name: url.lastPathComponent,
      path: url.path,
      isDir: values.isDirectory ?? false,
      size: values.fileSize.map { UInt64($0) },
      mtime: values.contentModificationDate.map { UInt64($0.timeIntervalSince1970 * 1000) }
    )
  }

  // ── Picker construction ───────────────────────────────────────────

  @available(iOS 14.0, *)
  private func textTypes() -> [UTType] {
    var types: [UTType] = [.plainText, .text, .data]
    if let md = UTType(filenameExtension: "md") {
      types.append(md)
    }
    if let markdown = UTType(filenameExtension: "markdown") {
      types.append(markdown)
    }
    return Array(Set(types))
  }

  private func legacyTextTypeIdentifiers() -> [String] {
    Array(Set([
      "public.plain-text",
      "public.text",
      "public.data",
      "net.daringfireball.markdown",
      "public.content",
    ]))
  }

  private func legacyFolderTypeIdentifiers() -> [String] {
    ["public.folder"]
  }

  private func makeFilePicker() -> UIDocumentPickerViewController {
    let picker: UIDocumentPickerViewController

    if #available(iOS 14.0, *) {
      picker = UIDocumentPickerViewController(
        forOpeningContentTypes: textTypes(),
        asCopy: false
      )
    } else {
      picker = UIDocumentPickerViewController(documentTypes: legacyTextTypeIdentifiers(), in: .open)
    }

    return configurePicker(picker)
  }

  private func makeFolderPicker() -> UIDocumentPickerViewController {
    let picker: UIDocumentPickerViewController

    if #available(iOS 14.0, *) {
      picker = UIDocumentPickerViewController(
        forOpeningContentTypes: [.folder],
        asCopy: false
      )
    } else {
      picker = UIDocumentPickerViewController(documentTypes: legacyFolderTypeIdentifiers(), in: .open)
    }

    return configurePicker(picker)
  }

  private func configurePicker(_ picker: UIDocumentPickerViewController) -> UIDocumentPickerViewController {
    picker.delegate = self
    picker.allowsMultipleSelection = false
    picker.modalPresentationStyle = .fullScreen
    picker.presentationController?.delegate = self
    return picker
  }

  // ── Path helpers ──────────────────────────────────────────────────

  private func initialDirectoryUrl(forSuggestedName suggestedName: String?) -> URL? {
    guard let suggestedName else {
      return nil
    }

    let directoryPath = (suggestedName as NSString).deletingLastPathComponent
    guard !directoryPath.isEmpty else {
      return nil
    }

    return URL(fileURLWithPath: directoryPath, isDirectory: true)
  }

  private func initialDirectoryUrl(forSuggestedNameFolder suggestedName: String?) -> URL? {
    guard let suggestedName, !suggestedName.isEmpty else {
      return nil
    }

    return URL(fileURLWithPath: suggestedName, isDirectory: true)
  }

  private func resolveScopedFolderUrl(id: String) throws -> URL {
    guard let bookmarkData = store.getBookmarkData(id: id) else {
      throw bookmarkError(.notFound, "No folder bookmark found for id: \(id)")
    }

    var stale = false
    let folderUrl = try resolveSecurityScopedBookmarkUrl(bookmarkData, stale: &stale)

    guard folderUrl.startAccessingSecurityScopedResource() else {
      throw bookmarkError(.permissionDenied, "Failed to access security-scoped folder for id: \(id)")
    }

    if stale {
      let refreshedBookmark = try makeSecurityScopedBookmarkData(for: folderUrl)
      let refreshedName = try? folderUrl.resourceValues(forKeys: [.nameKey]).name
      store.updateFolder(id: id, bookmarkData: refreshedBookmark, folderName: refreshedName)
    }

    return folderUrl
  }

  private func resolveScopedChildUrl(targetPath: String, folderUrl: URL) throws -> URL {
    try resolveScopedChildUrlInBookmarkScope(targetPath: targetPath, folderUrl: folderUrl)
  }

  private func assertFolderIsEmpty(url: URL) throws {
    let entries = try coordinatedList(url: url)
    guard entries.isEmpty else {
      throw bookmarkError(.folderNotEmpty, "Selected folder must be empty")
    }
  }

  private func sanitizeWorkspaceComponent(_ name: String) throws -> String {
    let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
      throw bookmarkError(.nativeError, "Name is required")
    }

    if trimmed.contains("/") || trimmed.contains("\\") || trimmed == "." || trimmed == ".." {
      throw bookmarkError(.nativeError, "Name is not valid")
    }

    return trimmed
  }

  // ── Bookmark data helpers ─────────────────────────────────────────

  private func makeSecurityScopedBookmarkData(for url: URL) throws -> Data {
    try url.bookmarkData(
      options: [],
      includingResourceValuesForKeys: nil,
      relativeTo: nil
    )
  }

  private func resolveSecurityScopedBookmarkUrl(_ bookmarkData: Data, stale: inout Bool) throws -> URL {
    try URL(
      resolvingBookmarkData: bookmarkData,
      options: [],
      relativeTo: nil,
      bookmarkDataIsStale: &stale
    )
  }

  private func rejectPendingInvoke(_ error: Error) {
    pendingInvoke?.reject(bookmarkRejectMessage(for: error))
    clearPendingPickState()
  }
}

#else

import Foundation

final class BookmarkPlugin: Plugin {}

#endif
