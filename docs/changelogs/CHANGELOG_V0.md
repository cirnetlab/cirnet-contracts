# 0.3.0-dev.1

## English

- Migrated the Rust crate, canonical schemas and catalogs, Hardware RPC and NCAP, TypeScript generation and conformance fixtures.
- Added Windows native packaging and SDK verification with source, crate, descriptor and artifact SHA-256 records.
- Fixed Windows CI packaging of the development artifact allocator by passing its directory directly to npm.
- Restored installed legacy wheel parity and the development allocator’s Python bridge regression checks.
- Replaced the installed wheel dependency with frozen Contracts 0.1.2 outcomes and a normalized Hardware RPC descriptor. Windows CI exercises the same parity cases.
- Updated the contract ownership references to the Ecosystem and CIRNET Lab-First architectures.
- Aligned the active manifest with the native NCAP source and current projection paths. Documented the owners of catalog inputs, generated resources, SDK packaging, developer tools and parity fixtures.

## Tiếng Việt

- Chuyển crate Rust, schema và danh mục chuẩn, Hardware RPC và NCAP, bộ sinh TypeScript và fixture tuân thủ.
- Thêm đóng gói native Windows và kiểm chứng SDK với mã băm SHA-256 của mã nguồn, crate, descriptor và artifact.
- Sửa bước đóng gói công cụ cấp phát artifact dành cho phát triển trên Windows CI bằng cách truyền trực tiếp thư mục gói cho npm.
- Khôi phục kiểm thử đối chiếu với wheel cũ đã cài đặt và kiểm thử hồi quy cầu nối Python của công cụ cấp phát artifact.
- Thay phụ thuộc vào wheel đã cài đặt bằng kết quả đã chốt từ Contracts 0.1.2 và descriptor Hardware RPC đã chuẩn hóa. CI Windows chạy cùng các ca đối chiếu.
- Cập nhật tham chiếu quyền sở hữu hợp đồng sang kiến trúc Ecosystem và CIRNET Lab-First.
- Cập nhật manifest hiện hành theo nguồn NCAP native và các đường dẫn bản chiếu hiện tại. Ghi rõ bên quản lý đầu vào danh mục, tài nguyên sinh tự động, bước đóng gói SDK, công cụ phát triển và fixture đối chiếu.
