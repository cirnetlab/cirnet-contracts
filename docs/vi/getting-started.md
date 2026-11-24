# Bắt đầu với CIRNET Contracts

[English](../en/getting-started.md) · [Tài liệu](index.md)

Crate Rust `cirnet-contracts-native` xác thực schema, cung cấp kiểu Hardware RPC và mã hóa NCAP. Tệp thực thi `cirnet-contracts` xác thực JSON, xuất descriptor protobuf và sinh đầu vào SDK. [SDK công khai](../../packages/public-sdk/README.md) cung cấp DTO, trình xác thực, siêu dữ liệu thao tác, tập schema đầy đủ và fixture tuân thủ.

Dùng [hướng dẫn build native](native.md) để build phiên bản `0.3.0-dev.1` với bộ công cụ Rust và lockfile npm đã ghim. Dùng đúng bản OpenAPI do Remote xuất và crate Contracts tương ứng để sinh SDK có thể tái lập. [Công cụ cấp phát artifact](../../tools/artifact-paths/README.md) là gói phát triển riêng cho đầu ra build và kiểm thử ngoài kho.

[Tài liệu tương thích](api/compatibility.md) ghi nhận định danh định dạng truyền và gói.
