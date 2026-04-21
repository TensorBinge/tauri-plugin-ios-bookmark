#if os(iOS)
import Foundation
#if os(iOS)
import Tauri
#endif
import PDFKit
import UIKit
import UniformTypeIdentifiers

private struct PickBookmarkArgs: Decodable {
  let request: PickBookmarkRequestDTO?
}

private struct PickFolderBookmarkArgs: Decodable {
  let request: PickFolderBookmarkRequestDTO?
}

private struct PickBookmarkRequestDTO: Decodable {
  let targetPath: String?
  let suggestedFileName: String?
}

private struct PickFolderBookmarkRequestDTO: Decodable {
  let targetPath: String?
  let requireEmpty: Bool?
}

private struct ExportTocEntryDTO: Decodable {
  let id: String
  let title: String
  let depth: Int
}

private enum PendingPickerKind {
  case file
  case folder
  case export
}

private final class PdfPrintPageRenderer: UIPrintPageRenderer {
  private let exportPaperRect: CGRect
  private let exportPrintableRect: CGRect

  init(paperRect: CGRect, printableRect: CGRect) {
    self.exportPaperRect = paperRect
    self.exportPrintableRect = printableRect
    super.init()
  }

  override var paperRect: CGRect {
    exportPaperRect
  }

  override var printableRect: CGRect {
    exportPrintableRect
  }
}

final class BookmarkPlugin: Plugin, UIDocumentPickerDelegate, UIAdaptivePresentationControllerDelegate {
  private var pendingInvoke: Invoke?
  private var pendingPickRequest: PickBookmarkRequestDTO?
  private var pendingFolderPickRequest: PickFolderBookmarkRequestDTO?
  private var pendingPickerKind: PendingPickerKind?
  private var pendingExportUrl: URL?
  private let store = BookmarkStore()

  @objc public func pickAndBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: pickAndBookmark entered mainThread=\(Thread.isMainThread)", category: "ios-bookmark")

    let args = (try? invoke.parseArgs(PickBookmarkArgs.self)) ?? PickBookmarkArgs(request: nil)

    let presentPicker = {
      Logger.info("[ios-bookmark] swift plugin: pickAndBookmark on main thread", category: "ios-bookmark")
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

      if let directoryUrl = self.initialDirectoryUrl(for: args.request) {
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

  @objc public func pickFolderAndBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: pickFolderAndBookmark entered mainThread=\(Thread.isMainThread)", category: "ios-bookmark")

    let args = (try? invoke.parseArgs(PickFolderBookmarkArgs.self)) ?? PickFolderBookmarkArgs(request: nil)

    let presentPicker = {
      Logger.info("[ios-bookmark] swift plugin: pickFolderAndBookmark on main thread", category: "ios-bookmark")
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

      if let directoryUrl = self.initialDirectoryUrl(for: args.request) {
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

  @objc public func exportFile(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: exportFile entered mainThread=\(Thread.isMainThread)", category: "ios-bookmark")

    struct Args: Decodable {
      let path: String
    }

    let presentPicker = {
      Logger.info("[ios-bookmark] swift plugin: exportFile on main thread", category: "ios-bookmark")
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

      do {
        let args = try invoke.parseArgs(Args.self)
        let fileUrl = URL(fileURLWithPath: args.path)
        Logger.info("[ios-bookmark] swift plugin: exportFile parsed path=\(fileUrl.path)", category: "ios-bookmark")
        Logger.info("[ios-bookmark] swift plugin: exportFile fileExists=\(FileManager.default.fileExists(atPath: fileUrl.path))", category: "ios-bookmark")
        guard FileManager.default.fileExists(atPath: fileUrl.path) else {
          throw bookmarkError(.notFound, "Export file not found")
        }

        Logger.info("[ios-bookmark] swift plugin: exportFile presenter=\(String(describing: type(of: presenter))) pendingInvoke=\(self.pendingInvoke != nil)", category: "ios-bookmark")

        let picker: UIDocumentPickerViewController
        if #available(iOS 14.0, *) {
          Logger.info("[ios-bookmark] swift plugin: exportFile creating iOS14+ export picker", category: "ios-bookmark")
          picker = UIDocumentPickerViewController(forExporting: [fileUrl], asCopy: true)
        } else {
          Logger.info("[ios-bookmark] swift plugin: exportFile creating legacy export picker", category: "ios-bookmark")
          picker = UIDocumentPickerViewController(url: fileUrl, in: .exportToService)
        }

        self.pendingInvoke = invoke
        self.pendingPickerKind = .export
        self.pendingExportUrl = fileUrl

        Logger.info("[ios-bookmark] swift plugin: presenting export UIDocumentPickerViewController", category: "ios-bookmark")
        presenter.present(self.configurePicker(picker), animated: true) {
          Logger.info("[ios-bookmark] swift plugin: export UIDocumentPickerViewController present completion", category: "ios-bookmark")
        }
      } catch {
        Logger.error("[ios-bookmark] swift plugin: exportFile error \(detailedErrorDescription(error))", category: "ios-bookmark")
        invoke.reject(bookmarkRejectMessage(for: error))
      }
    }

    if Thread.isMainThread {
      presentPicker()
    } else {
      DispatchQueue.main.async(execute: presentPicker)
    }
  }

  @objc public func exportPdf(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: exportPdf entered mainThread=\(Thread.isMainThread)", category: "ios-bookmark")

    struct Args: Decodable {
      let fileName: String
      let html: String
      let toc: [ExportTocEntryDTO]?
    }

    let presentPicker = {
      Logger.info("[ios-bookmark] swift plugin: exportPdf on main thread", category: "ios-bookmark")

      do {
        let args = try invoke.parseArgs(Args.self)
        let toc = args.toc ?? []
        Logger.info("[ios-bookmark] swift plugin: exportPdf parsed fileName=\(args.fileName) htmlLength=\(args.html.count) tocLength=\(toc.count)", category: "ios-bookmark")
        let fileUrl = try self.renderPdfExportFile(fileName: args.fileName, html: args.html, toc: toc)
        Logger.info("[ios-bookmark] swift plugin: exportPdf rendered temp file at \(fileUrl.path)", category: "ios-bookmark")
        try self.presentExportPicker(fileUrl: fileUrl, invoke: invoke)
      } catch {
        Logger.error("[ios-bookmark] swift plugin: exportPdf error \(detailedErrorDescription(error))", category: "ios-bookmark")
        invoke.reject(bookmarkRejectMessage(for: error))
      }
    }

    if Thread.isMainThread {
      presentPicker()
    } else {
      DispatchQueue.main.async(execute: presentPicker)
    }
  }

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
    case .export:
      resolveExport(urls.first)
    case .folder:
      resolvePickedFolder(url)
    case .file, .none:
      resolvePickedFile(url)
    }
  }

  @objc public func readByFolderBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: readByFolderBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let targetPath: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      guard let bookmarkData = store.getBookmarkData(id: args.id) else {
        throw bookmarkError(.notFound, "No folder bookmark found for id: \(args.id)")
      }

      var stale = false
      let folderUrl = try resolveSecurityScopedBookmarkUrl(bookmarkData, stale: &stale)

      guard urlIsWithinFolder(targetPath: args.targetPath, folderPath: folderUrl.path) else {
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

      let targetUrl = try resolveScopedChildUrl(targetPath: args.targetPath, folderUrl: folderUrl)
      let content = try coordinatedRead(url: targetUrl)
      Logger.info("[ios-bookmark] swift plugin: readByFolderBookmark resolved for \(targetUrl.lastPathComponent)", category: "ios-bookmark")
      invoke.resolve(ReadResultDTO(fileName: targetUrl.lastPathComponent, filePath: targetUrl.path, content: content))
    } catch {
      Logger.error("[ios-bookmark] swift plugin: readByFolderBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func listByFolderBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: listByFolderBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let targetPath: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      let folderUrl = try resolveScopedFolderUrl(id: args.id)
      defer { folderUrl.stopAccessingSecurityScopedResource() }
      let targetUrl = try resolveScopedChildUrl(targetPath: args.targetPath, folderUrl: folderUrl)
      let entries = try coordinatedList(url: targetUrl)
      invoke.resolve(entries)
    } catch {
      Logger.error("[ios-bookmark] swift plugin: listByFolderBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func createFolderByFolderBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: createFolderByFolderBookmark entered", category: "ios-bookmark")
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
      let name = try sanitizeWorkspaceComponent(args.name, requireMarkdownExtension: false)
      let entry = try coordinatedCreateFolder(parentUrl: parentUrl, name: name)
      invoke.resolve(entry)
    } catch {
      Logger.error("[ios-bookmark] swift plugin: createFolderByFolderBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func createMarkdownFileByFolderBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: createMarkdownFileByFolderBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let parentPath: String
      let name: String
      let content: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      let folderUrl = try resolveScopedFolderUrl(id: args.id)
      defer { folderUrl.stopAccessingSecurityScopedResource() }
      let parentUrl = try resolveScopedChildUrl(targetPath: args.parentPath, folderUrl: folderUrl)
      let name = try sanitizeWorkspaceComponent(args.name, requireMarkdownExtension: true)
      let entry = try coordinatedCreateMarkdownFile(parentUrl: parentUrl, name: name, content: args.content)
      invoke.resolve(entry)
    } catch {
      Logger.error("[ios-bookmark] swift plugin: createMarkdownFileByFolderBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func writeByFolderBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: writeByFolderBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
      let targetPath: String
      let content: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      guard let bookmarkData = store.getBookmarkData(id: args.id) else {
        throw bookmarkError(.notFound, "No folder bookmark found for id: \(args.id)")
      }

      var stale = false
      let folderUrl = try resolveSecurityScopedBookmarkUrl(bookmarkData, stale: &stale)

      guard urlIsWithinFolder(targetPath: args.targetPath, folderPath: folderUrl.path) else {
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

      let targetUrl = try resolveScopedChildUrl(targetPath: args.targetPath, folderUrl: folderUrl)
      try coordinatedWrite(url: targetUrl, content: args.content)
      Logger.info("[ios-bookmark] swift plugin: writeByFolderBookmark resolved for \(targetUrl.lastPathComponent)", category: "ios-bookmark")
      invoke.resolve()
    } catch {
      Logger.error("[ios-bookmark] swift plugin: writeByFolderBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  private func resolvePickedFile(_ url: URL) {
    if let targetPath = pendingPickRequest?.targetPath,
       !pathsMatch(selectedUrl: url, targetPath: targetPath) {
      Logger.error("[ios-bookmark] swift plugin: selected file does not match requested target", category: "ios-bookmark")
      rejectPendingInvoke(bookmarkError(.targetMismatch, "Selected file does not match requested target"))
      return
    }

    guard url.startAccessingSecurityScopedResource() else {
      Logger.error("[ios-bookmark] swift plugin: startAccessingSecurityScopedResource failed for \(url)", category: "ios-bookmark")
      rejectPendingInvoke(bookmarkError(.permissionDenied, "Failed to access security-scoped resource"))
      return
    }
    defer { url.stopAccessingSecurityScopedResource() }

    do {
      let bookmarkData = try makeSecurityScopedBookmarkData(for: url)
      let fileName = url.lastPathComponent
      let content = try coordinatedRead(url: url)
      let id = store.save(bookmarkData: bookmarkData, fileName: fileName)
      Logger.info("[ios-bookmark] swift plugin: resolving pickAndBookmark for \(fileName)", category: "ios-bookmark")

      pendingInvoke?.resolve(
        PickResultDTO(bookmarkId: id, fileName: fileName, filePath: url.path, content: content)
      )
    } catch {
      Logger.error("[ios-bookmark] swift plugin: pickAndBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      rejectPendingInvoke(error)
      return
    }

    clearPendingPickState()
  }

  private func resolvePickedFolder(_ url: URL) {
    if let targetPath = pendingFolderPickRequest?.targetPath,
       !pathsMatch(selectedUrl: url, targetPath: targetPath) {
      Logger.error("[ios-bookmark] swift plugin: selected folder does not match requested target", category: "ios-bookmark")
      rejectPendingInvoke(bookmarkError(.targetMismatch, "Selected folder does not match requested target"))
      return
    }

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
      Logger.info("[ios-bookmark] swift plugin: resolving pickFolderAndBookmark for \(folderName)", category: "ios-bookmark")

      pendingInvoke?.resolve(
        PickFolderResultDTO(bookmarkId: id, folderName: folderName, folderPath: resolvedFolderUrl.path)
      )
    } catch {
      Logger.error("[ios-bookmark] swift plugin: pickFolderAndBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      rejectPendingInvoke(error)
      return
    }

    clearPendingPickState()
  }

  private func resolveExport(_ url: URL?) {
    if let url {
      Logger.info("[ios-bookmark] swift plugin: exportFile completed to \(url.path)", category: "ios-bookmark")
    } else {
      Logger.info("[ios-bookmark] swift plugin: exportFile completed without a destination callback url", category: "ios-bookmark")
    }

    pendingInvoke?.resolve()
    clearPendingPickState()
  }

  private func renderPdfExportFile(fileName: String, html: String, toc: [ExportTocEntryDTO]) throws -> URL {
    let sanitizedFileName = try sanitizeExportFileName(fileName, expectedExtension: "pdf")
    let fileUrl = try makeTemporaryExportUrl(fileName: sanitizedFileName)
    let pageRect = CGRect(x: 0, y: 0, width: 595.2, height: 841.8)
    let printableRect = pageRect.insetBy(dx: 24, dy: 24)
    let renderer = PdfPrintPageRenderer(paperRect: pageRect, printableRect: printableRect)
    let formatter = UIMarkupTextPrintFormatter(markupText: html)

    renderer.addPrintFormatter(formatter, startingAtPageAt: 0)

    let pageCount = max(renderer.numberOfPages, 1)
    let pdfData = NSMutableData()
    UIGraphicsBeginPDFContextToData(pdfData, pageRect, nil)

    for pageIndex in 0..<pageCount {
      UIGraphicsBeginPDFPageWithInfo(pageRect, nil)
      renderer.drawPage(at: pageIndex, in: pageRect)
    }

    UIGraphicsEndPDFContext()
    let outlinedPdfData = addOutlineMetadata(to: pdfData as Data, toc: toc)
    try outlinedPdfData.write(to: fileUrl, options: .atomic)
    return fileUrl
  }

  private func addOutlineMetadata(to pdfData: Data, toc: [ExportTocEntryDTO]) -> Data {
    guard !toc.isEmpty else {
      return pdfData
    }

    guard let document = PDFDocument(data: pdfData) else {
      return pdfData
    }

    let root = PDFOutline()
    var outlineStack: [(depth: Int, outline: PDFOutline)] = [(depth: 0, outline: root)]
    var lastSelection: PDFSelection?

    for entry in toc {
      guard let selection = findSelection(for: entry.title, after: lastSelection, in: document) else {
        continue
      }

      guard let page = selection.pages.first else {
        continue
      }

      let bounds = selection.bounds(for: page)
      let outline = PDFOutline()
      outline.label = entry.title
      outline.destination = PDFDestination(page: page, at: CGPoint(x: bounds.minX, y: bounds.maxY))

      while outlineStack.count > 1 && entry.depth <= outlineStack[outlineStack.count - 1].depth {
        outlineStack.removeLast()
      }

      let parentOutline = outlineStack[outlineStack.count - 1].outline
      parentOutline.insertChild(outline, at: parentOutline.numberOfChildren)
      outlineStack.append((depth: max(entry.depth, 1), outline: outline))
      lastSelection = selection
    }

    guard root.numberOfChildren > 0 else {
      return pdfData
    }

    document.outlineRoot = root
    return document.dataRepresentation() ?? pdfData
  }

  private func findSelection(for title: String, after previousSelection: PDFSelection?, in document: PDFDocument) -> PDFSelection? {
    if let previousSelection {
      if let nextSelection = document.findString(title, fromSelection: previousSelection, withOptions: []) {
        return nextSelection
      }

      if let nextSelection = document.findString(title, fromSelection: previousSelection, withOptions: [.caseInsensitive, .diacriticInsensitive]) {
        return nextSelection
      }
    }

    if let exactSelection = document.findString(title, withOptions: []).first {
      return exactSelection
    }

    return document.findString(title, withOptions: [.caseInsensitive, .diacriticInsensitive]).first
  }

  private func presentExportPicker(fileUrl: URL, invoke: Invoke) throws {
    guard let presenter = self.manager.viewController else {
      Logger.error("[ios-bookmark] swift plugin: no active view controller", category: "ios-bookmark")
      throw bookmarkError(.nativeError, "No active view controller")
    }

    guard self.pendingInvoke == nil else {
      Logger.error("[ios-bookmark] swift plugin: picker already in progress", category: "ios-bookmark")
      throw bookmarkError(.nativeError, "Picker already in progress")
    }

    Logger.info("[ios-bookmark] swift plugin: exportFile parsed path=\(fileUrl.path)", category: "ios-bookmark")
    Logger.info("[ios-bookmark] swift plugin: exportFile fileExists=\(FileManager.default.fileExists(atPath: fileUrl.path))", category: "ios-bookmark")
    guard FileManager.default.fileExists(atPath: fileUrl.path) else {
      throw bookmarkError(.notFound, "Export file not found")
    }

    Logger.info("[ios-bookmark] swift plugin: exportFile presenter=\(String(describing: type(of: presenter))) pendingInvoke=\(self.pendingInvoke != nil)", category: "ios-bookmark")

    let picker: UIDocumentPickerViewController
    if #available(iOS 14.0, *) {
      Logger.info("[ios-bookmark] swift plugin: exportFile creating iOS14+ export picker", category: "ios-bookmark")
      picker = UIDocumentPickerViewController(forExporting: [fileUrl], asCopy: true)
    } else {
      Logger.info("[ios-bookmark] swift plugin: exportFile creating legacy export picker", category: "ios-bookmark")
      picker = UIDocumentPickerViewController(url: fileUrl, in: .exportToService)
    }

    self.pendingInvoke = invoke
    self.pendingPickerKind = .export
    self.pendingExportUrl = fileUrl

    Logger.info("[ios-bookmark] swift plugin: presenting export UIDocumentPickerViewController", category: "ios-bookmark")
    presenter.present(self.configurePicker(picker), animated: true) {
      Logger.info("[ios-bookmark] swift plugin: export UIDocumentPickerViewController present completion", category: "ios-bookmark")
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

  @objc public func readByBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: readByBookmark entered", category: "ios-bookmark")
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
      Logger.info("[ios-bookmark] swift plugin: readByBookmark resolved for \(fileName)", category: "ios-bookmark")
      invoke.resolve(ReadResultDTO(fileName: fileName, filePath: url.path, content: content))
    } catch {
      Logger.error("[ios-bookmark] swift plugin: readByBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func writeByBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: writeByBookmark entered", category: "ios-bookmark")
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
      Logger.info("[ios-bookmark] swift plugin: writeByBookmark resolved for \(url.lastPathComponent)", category: "ios-bookmark")
      invoke.resolve()
    } catch {
      Logger.error("[ios-bookmark] swift plugin: writeByBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

  @objc public func forgetBookmark(_ invoke: Invoke) {
    Logger.info("[ios-bookmark] swift plugin: forgetBookmark entered", category: "ios-bookmark")
    struct Args: Decodable {
      let id: String
    }

    do {
      let args = try invoke.parseArgs(Args.self)
      store.remove(id: args.id)
      Logger.info("[ios-bookmark] swift plugin: forgetBookmark resolved", category: "ios-bookmark")
      invoke.resolve()
    } catch {
      Logger.error("[ios-bookmark] swift plugin: forgetBookmark error \(detailedErrorDescription(error))", category: "ios-bookmark")
      invoke.reject(bookmarkRejectMessage(for: error))
    }
  }

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

  private func coordinatedList(url: URL) throws -> [FolderBookmarkEntryDTO] {
    var entries: [FolderBookmarkEntryDTO] = []
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
          return FolderBookmarkEntryDTO(
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

  private func coordinatedCreateFolder(parentUrl: URL, name: String) throws -> FolderBookmarkEntryDTO {
    var createdEntry: FolderBookmarkEntryDTO?
    var coordinatorError: NSError?
    var createError: Error?
    let coordinator = NSFileCoordinator(filePresenter: nil)

    coordinator.coordinate(writingItemAt: parentUrl, options: .forMerging, error: &coordinatorError) { writeUrl in
      do {
        let folderUrl = writeUrl.appendingPathComponent(name, isDirectory: true)
        try FileManager.default.createDirectory(at: folderUrl, withIntermediateDirectories: false, attributes: nil)
        createdEntry = FolderBookmarkEntryDTO(name: name, path: folderUrl.path, isDir: true, size: 0, mtime: UInt64(Date().timeIntervalSince1970 * 1000))
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

  private func coordinatedCreateMarkdownFile(parentUrl: URL, name: String, content: String) throws -> FolderBookmarkEntryDTO {
    var createdEntry: FolderBookmarkEntryDTO?
    var coordinatorError: NSError?
    var createError: Error?
    let coordinator = NSFileCoordinator(filePresenter: nil)

    coordinator.coordinate(writingItemAt: parentUrl, options: .forMerging, error: &coordinatorError) { writeUrl in
      do {
        let fileUrl = writeUrl.appendingPathComponent(name, isDirectory: false)
        try content.write(to: fileUrl, atomically: true, encoding: .utf8)
        createdEntry = FolderBookmarkEntryDTO(name: name, path: fileUrl.path, isDir: false, size: UInt64(content.lengthOfBytes(using: .utf8)), mtime: UInt64(Date().timeIntervalSince1970 * 1000))
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
      throw bookmarkError(.ioError, "Failed to create markdown file")
    }

    return createdEntry
  }

  @available(iOS 14.0, *)
  private func markdownTypes() -> [UTType] {
    var types: [UTType] = [.plainText, .text]
    if let markdown = UTType(filenameExtension: "md") {
      types.append(markdown)
    }
    if let markdownAlt = UTType(filenameExtension: "markdown") {
      types.append(markdownAlt)
    }
    return Array(Set(types))
  }

  private func legacyMarkdownTypeIdentifiers() -> [String] {
    Array(Set([
      "public.plain-text",
      "public.text",
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
        forOpeningContentTypes: markdownTypes(),
        asCopy: false
      )
    } else {
      picker = UIDocumentPickerViewController(documentTypes: legacyMarkdownTypeIdentifiers(), in: .open)
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

  private func sanitizeExportFileName(_ fileName: String, expectedExtension: String) throws -> String {
    let sanitized = (fileName as NSString).lastPathComponent.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !sanitized.isEmpty else {
      throw bookmarkError(.nativeError, "Export file name is required")
    }

    if (sanitized as NSString).pathExtension.lowercased() != expectedExtension {
      return ((sanitized as NSString).deletingPathExtension as NSString).appendingPathExtension(expectedExtension) ?? sanitized + ".\(expectedExtension)"
    }

    return sanitized
  }

  private func makeTemporaryExportUrl(fileName: String) throws -> URL {
    let exportDirectory = FileManager.default.temporaryDirectory.appendingPathComponent("markscope-exports", isDirectory: true)
    try FileManager.default.createDirectory(at: exportDirectory, withIntermediateDirectories: true)
    let scopedExportDirectory = exportDirectory.appendingPathComponent("\(DispatchTime.now().uptimeNanoseconds)", isDirectory: true)
    try FileManager.default.createDirectory(at: scopedExportDirectory, withIntermediateDirectories: true)
    return scopedExportDirectory.appendingPathComponent(fileName)
  }

  private func initialDirectoryUrl(for request: PickBookmarkRequestDTO?) -> URL? {
    guard let targetPath = request?.targetPath else {
      return nil
    }

    let directoryPath = (targetPath as NSString).deletingLastPathComponent
    guard !directoryPath.isEmpty else {
      return nil
    }

    return URL(fileURLWithPath: directoryPath, isDirectory: true)
  }

  private func initialDirectoryUrl(for request: PickFolderBookmarkRequestDTO?) -> URL? {
    guard let targetPath = request?.targetPath, !targetPath.isEmpty else {
      return nil
    }

    return URL(fileURLWithPath: targetPath, isDirectory: true)
  }

  private func pathsMatch(selectedUrl: URL, targetPath: String) -> Bool {
    standardizedPath(for: selectedUrl.path) == standardizedPath(for: targetPath)
  }

  private func standardizedPath(for path: String) -> String {
    normalizeIosScopedPath(URL(fileURLWithPath: path).standardizedFileURL.path)
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

  private func sanitizeWorkspaceComponent(_ name: String, requireMarkdownExtension: Bool) throws -> String {
    let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
      throw bookmarkError(.nativeError, "Name is required")
    }

    if trimmed.contains("/") || trimmed.contains("\\") || trimmed == "." || trimmed == ".." {
      throw bookmarkError(.nativeError, "Name is not valid")
    }

    if requireMarkdownExtension {
      if trimmed.lowercased().hasSuffix(".md") {
        return trimmed
      }

      if trimmed.lowercased().hasSuffix(".markdown") {
        return trimmed
      }

      return trimmed + ".md"
    }

    return trimmed
  }

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

  private func resolvePendingCancellation(reason: String) {
    Logger.info("[ios-bookmark] swift plugin: resolving picker cancellation kind=\(String(describing: pendingPickerKind)) reason=\(reason)", category: "ios-bookmark")

    switch pendingPickerKind {
    case .file:
      pendingInvoke?.resolve(Optional<PickResultDTO>.none)
    case .folder:
      pendingInvoke?.resolve(Optional<PickFolderResultDTO>.none)
    case .export, .none:
      pendingInvoke?.resolve()
    }

    clearPendingPickState()
  }

  private func clearPendingPickState() {
    cleanupPendingExportFile()
    pendingInvoke = nil
    pendingPickRequest = nil
    pendingFolderPickRequest = nil
    pendingPickerKind = nil
    pendingExportUrl = nil
  }

  private func cleanupPendingExportFile() {
    guard let pendingExportUrl else {
      return
    }

    let tempRoot = FileManager.default.temporaryDirectory.standardizedFileURL.path
    let exportPath = pendingExportUrl.standardizedFileURL.path

    guard exportPath.hasPrefix(tempRoot) else {
      return
    }

    try? FileManager.default.removeItem(at: pendingExportUrl)
  }
}

#else

import Foundation

final class BookmarkPlugin: Plugin {}

#endif
