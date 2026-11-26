# Kiểm chứng CIRNET Contracts

[English](../en/verification.md) · [Tài liệu](index.md)

## Cấp phát phiên

Đặt `CIRNET_ARTIFACTS_DIR` thành đường dẫn tuyệt đối bên ngoài mọi checkout, worktree và thư mục Git tham gia. Công cụ Node tạo một phiên có chủ sở hữu ở đó. Với kiểm chứng nhiều kho, đặt `CIRNET_ARTIFACTS_CONTEXT` để khai báo chủ phiên và các thư mục gốc tham gia. Xem [hướng dẫn công cụ cấp phát](../../tools/artifact-paths/README.md).

## Chạy kiểm tra

Chạy `cargo fmt --all --check`, `cargo test --locked` và bộ kiểm thử hồi quy SDK theo [hướng dẫn build native](native.md). Workflow Windows kiểm chứng gói native, crate, descriptor RPC và gói npm được sinh. Từ thư mục gốc của checkout, workflow đóng gói công cụ cấp phát dành cho phát triển bằng `npm pack ./tools/artifact-paths --ignore-scripts --pack-destination <sdk-output>`. Ứng viên hệ thống còn dùng đúng bản Remote xuất và ghi mã băm mã nguồn, manifest và crate Contracts.

## Kiểm chứng tính tương thích với kết quả tham chiếu đã chốt

Từ checkout Contracts, dùng Windows PowerShell 7, CPython 3.14.4 x64 và bộ công cụ Rust đã chốt. Cấp phát và xuất một `CIRNET_ARTIFACTS_SESSION` mới, sau đó chạy cùng phép đối chiếu với workflow Windows.

~~~powershell
./scripts/test-native-parity.ps1
~~~

Để chạy ngoại tuyến, cung cấp `-Wheelhouse <path>` trỏ đến kho wheel cục bộ chứa các gói bên thứ ba đã khóa. Trỏ `CARGO_HOME` đến bộ nhớ đệm chứa các crate Rust đã khóa và đặt `CARGO_NET_OFFLINE=true` trong môi trường build.

146 ca kiểm tra hồ sơ xác thực, biên số học, giải mã và lỗi NCAP, mã hóa lại NCAP theo dạng chuẩn và thời điểm của frame, cùng descriptor Hardware RPC. [Dữ liệu kiểm chứng](../../tests/fixtures/README.md) ghi các kết quả mong đợi từ Contracts 0.1.2 và nguồn gốc của chúng. Xem mã băm của dữ liệu kiểm thử và binding trong tệp `native-parity/inputs.json` của phiên. Trình chạy chỉ bật `python-tests` cho phép đối chiếu này.

## Kiểm tra công cụ cấp phát dành cho phát triển

Workflow Windows cũng chạy kiểm thử công cụ cấp phát Node và cầu nối Python. Đặt `ALLOCATOR_TEST_SOURCE` thành checkout Contracts, `ALLOCATOR_TEST_SCRATCH` thành thư mục trong phiên thuộc sở hữu và `PYTHONPATH` thành đường dẫn tuyệt đối của `tools/artifact-paths` để chạy các kiểm tra sau tại máy cục bộ:

~~~powershell
node --test tools/artifact-paths/test/artifact-paths.test.mjs
python -B -m unittest discover -s tools/artifact-paths/test -p test_python_bridge.py
~~~

## Rà soát và dọn dẹp

Lưu đầu ra thô trong phiên bên ngoài kho. Rà soát manifest và log kiểm thử, sau đó ghi bản tóm tắt đã duyệt cùng định danh artifact chính xác. Khi hoàn tất, xóa phiên thuộc sở hữu bằng `node tools/artifact-paths/artifact-paths.mjs --cleanup <session>`.

## Lưu trữ phát triển cục bộ

Đặt `CIRNET_LOCAL_STORAGE_ROOT` thành thư mục cục bộ hiện hữu ngoài repo để cấp phát theo định danh GitHub chính tắc. Thứ tự chọn thư mục cha là giá trị chỉ định, `CIRNET_ARTIFACTS_DIR`, namespace của repo, rồi thư mục tạm hệ điều hành. Thư mục gốc cấu hình vẫn được kiểm tra khi ghi đè. Phiên thừa kế phải thuộc thư mục cha đã chọn. Worktree dùng chung namespace và có quyền sở hữu phiên riêng.
