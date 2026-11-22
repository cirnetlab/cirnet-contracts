# Kiểm chứng CIRNET Contracts

[English](../en/verification.md) · [Tài liệu](index.md)

## Cấp phát phiên

Đặt `CIRNET_ARTIFACTS_DIR` thành đường dẫn tuyệt đối bên ngoài mọi checkout, worktree và thư mục Git tham gia. Công cụ Node tạo một phiên có chủ sở hữu ở đó. Với kiểm chứng nhiều kho, đặt `CIRNET_ARTIFACTS_CONTEXT` để khai báo chủ phiên và các thư mục gốc tham gia. Xem [hướng dẫn công cụ cấp phát](../../tools/artifact-paths/README.md).

## Chạy kiểm tra

Chạy `cargo fmt --all --check`, `cargo test --locked` và bộ kiểm thử hồi quy SDK theo [hướng dẫn build native](native.md). Workflow Windows kiểm chứng gói native, crate, descriptor RPC và gói npm được sinh. Ứng viên hệ thống còn dùng đúng bản Remote xuất và ghi mã băm mã nguồn, manifest và crate Contracts.

## Rà soát và dọn dẹp

Lưu đầu ra thô trong phiên bên ngoài kho. Rà soát manifest và log kiểm thử, sau đó ghi bản tóm tắt đã duyệt cùng định danh artifact chính xác. Khi hoàn tất, xóa phiên thuộc sở hữu bằng `node tools/artifact-paths/artifact-paths.mjs --cleanup <session>`.
