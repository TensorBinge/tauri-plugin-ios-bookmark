// swift-tools-version:5.3
import PackageDescription

let package = Package(
  name: "tauri-plugin-ios-bookmark",
  platforms: [
    .iOS(.v14),
  ],
  products: [
    .library(
      name: "tauri-plugin-ios-bookmark",
      type: .static,
      targets: ["tauri-plugin-ios-bookmark"]
    )
  ],
  dependencies: [
    .package(name: "Tauri", path: "../.tauri/tauri-api")
  ],
  targets: [
    .target(
      name: "tauri-plugin-ios-bookmark",
      dependencies: [
        .product(name: "Tauri", package: "Tauri")
      ],
      path: "Sources"
    ),
    .testTarget(
      name: "tauri-plugin-ios-bookmarkTests",
      dependencies: ["tauri-plugin-ios-bookmark"],
      path: "Tests"
    )
  ]
)
