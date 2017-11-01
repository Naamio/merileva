// swift-tools-version:4.0

import PackageDescription

let package = Package(
    name: "ExampleClient",
    dependencies: [
        .package(url: "../Merileva", from: "1.0.0")
    ],
    targets: [
        .target(
            name: "ExampleClient",
            dependencies: []),
    ]
)
