# Kiểm chứng CIRNET Contracts

[English](../en/verification.md) · [Tài liệu](index.md)

## Cấp phát phiên

Đặt `CIRNET_ARTIFACTS_DIR` thành đường dẫn tuyệt đối bên ngoài mọi checkout, worktree và thư mục Git tham gia. Công cụ Node tạo một phiên có chủ sở hữu ở đó. Với kiểm chứng nhiều kho, đặt `CIRNET_ARTIFACTS_CONTEXT` để khai báo chủ phiên và các thư mục gốc tham gia. Xem [hướng dẫn công cụ cấp phát](../../tools/artifact-paths/README.md).

## Chạy kiểm tra

Chạy `cargo fmt --all --check`, `cargo test --locked` và bộ kiểm thử hồi quy SDK theo [hướng dẫn build native](native.md). Workflow Windows kiểm chứng gói native, crate, descriptor RPC và gói npm được sinh. Từ thư mục gốc của checkout, workflow đóng gói công cụ cấp phát dành cho phát triển bằng `npm pack ./tools/artifact-paths --ignore-scripts --pack-destination <sdk-output>`. Ứng viên hệ thống còn dùng đúng bản Remote xuất và ghi mã băm mã nguồn, manifest và crate Contracts.

## Đối chiếu với bản triển khai cũ đã cài đặt

Từ checkout Next, dùng Windows PowerShell 7, CPython 3.14.4 x64 và bộ công cụ Rust đã chốt. Khai báo cả hai checkout Contracts trong `CIRNET_ARTIFACTS_CONTEXT` trước khi cấp phát và xuất một `CIRNET_ARTIFACTS_SESSION` mới. Chạy:

~~~powershell
./scripts/test-legacy-parity.ps1 -LegacyCheckout ../cirnet-contracts
~~~

Trình chạy xác minh cây Git ở Contracts 0.1.2, tạo bản lưu trong phiên và build wheel tham chiếu tại đó. Dùng `-SealedWheel <path>` thay cho `-LegacyCheckout` để đối chiếu với wheel phát hành gốc. Trình chạy kiểm tra mã SHA-256 đã chốt của wheel này. Tùy chọn `-Wheelhouse <path>` dùng kho wheel cục bộ cho các phụ thuộc đã khóa.

Các ca đối chiếu đã khôi phục so sánh hồ sơ kiểm tra, biên số học, byte và thời gian NCAP, cùng descriptor Hardware RPC với wheel đã cài đặt. Xem mã băm của bản tham chiếu và binding trong `legacy-parity/inputs.json`. Wheel build lại là bản tham chiếu từ mã nguồn. Bản build sản xuất vẫn tắt tính năng `python-tests`.

Workflow Windows cũng chạy kiểm thử công cụ cấp phát Node và cầu nối Python. Đặt `ALLOCATOR_TEST_SOURCE` thành checkout Next, `ALLOCATOR_TEST_SCRATCH` thành thư mục trong phiên thuộc sở hữu và `PYTHONPATH` thành đường dẫn tuyệt đối của `tools/artifact-paths` để chạy các kiểm tra sau tại máy cục bộ:

~~~powershell
node --test tools/artifact-paths/test/artifact-paths.test.mjs
python -B -m unittest discover -s tools/artifact-paths/test -p test_python_bridge.py
~~~

## Rà soát và dọn dẹp

Lưu đầu ra thô trong phiên bên ngoài kho. Rà soát manifest và log kiểm thử, sau đó ghi bản tóm tắt đã duyệt cùng định danh artifact chính xác. Khi hoàn tất, xóa phiên thuộc sở hữu bằng `node tools/artifact-paths/artifact-paths.mjs --cleanup <session>`.
