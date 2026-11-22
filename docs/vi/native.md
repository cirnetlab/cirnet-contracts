# Build và kiểm chứng Contracts Rust

[English](../en/native.md) · Tiếng Việt · [Mục lục tài liệu](index.md)

Crate `cirnet-contracts-native` 0.3.0-dev.1 cung cấp trình xác thực ngoại tuyến, codec NCAP1 và NCAP2, kiểu dữ liệu và descriptor Hardware RPC chuẩn, cùng bộ sinh SDK TypeScript. Tệp thực thi có tên `cirnet-contracts`. Manifest, profile schema, thư viện thiết bị, bố cục capture, proto RPC và OpenAPI đã chốt là đầu vào dùng chung. [Tra cứu hợp đồng dữ liệu](contract-reference.md)

## Build và chạy

Môi trường phát triển dùng toolchain Rust và Cargo đã ghim ở phiên bản 1.99.1 cùng lockfile của kho mã. Crate khai báo Rust 1.90 là phiên bản trình biên dịch tối thiểu được hỗ trợ. [Bản ghim toolchain](../../rust-toolchain.toml) · [Manifest Cargo](../../Cargo.toml) · [Lockfile Cargo](../../Cargo.lock)

Trên Windows 64 bit, dùng host `x86_64-pc-windows-msvc` cùng Visual Studio C++ Build Tools và Windows SDK. Chạy `rustc -Vv` và `cargo -Vv` tại thư mục gốc của kho mã để kiểm tra phiên bản và host.

Nếu Rustup chỉ có bí danh `stable`, kiểm tra `rustup run stable rustc -vV` và `rustup run stable cargo -vV`. Dùng `rustup run stable cargo` cho các lệnh bên dưới khi bí danh trỏ tới 1.99.1 trên đúng host.

Gán `CIRNET_ARTIFACTS_SESSION` bằng phiên ngoài kho do bộ cấp phát quản lý rồi chạy các lệnh POSIX sau tại thư mục gốc của kho mã. Giữ nguyên shell để chạy các bước tiếp theo. [Cấp phát phiên](verification.md#cấp-phát-phiên) và [dọn phiên sau khi xem kết quả](verification.md#xem-kết-quả-và-dọn-phiên).

~~~sh
repo="$(pwd -P)"
test -n "$CIRNET_ARTIFACTS_SESSION"
export CARGO_TARGET_DIR="$CIRNET_ARTIFACTS_SESSION/cargo-target"
export TMP="$CIRNET_ARTIFACTS_SESSION" TEMP="$CIRNET_ARTIFACTS_SESSION" TMPDIR="$CIRNET_ARTIFACTS_SESSION"

cargo build --locked --release
cargo test --locked

printf '%s\n' '{"status":"ok","service":"cirnet-api","hardware_mode":"simulation"}' |
  cargo run --locked --release -- validate "$repo" HealthResponse transport
cargo run --locked --release -- descriptor > "$CIRNET_ARTIFACTS_SESSION/rpc-v2.pb"
~~~

`validate` nhận đường dẫn gốc của mã nguồn, tên hợp đồng và profile. Lệnh đọc JSON từ đầu vào chuẩn, ghi JSON đã kiểm tra ra đầu ra chuẩn và trả mã thoát 2 kèm lỗi có cấu trúc trên luồng lỗi chuẩn khi dữ liệu vi phạm hợp đồng. `descriptor` ghi descriptor protobuf chuẩn ở dạng nhị phân.

## Gói Windows native

Từ checkout Contracts trên PowerShell với toolchain MSVC đã ghim, cấp phát phiên kiểm chứng rồi build gói ứng viên.

~~~powershell
Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
$env:CIRNET_ARTIFACTS_SESSION = node tools/artifact-paths/artifact-paths.mjs
$bundle = Join-Path $env:CIRNET_ARTIFACTS_SESSION 'contracts-native'
node scripts/package-native.mjs $bundle
$receipt = Get-Content -Raw (Join-Path $bundle "contracts-native-manifest.json") | ConvertFrom-Json
$target = Join-Path $env:CIRNET_ARTIFACTS_SESSION "contracts-native-target"
node scripts/check-native-package.mjs $bundle (Join-Path $target ("package/" + $receipt.crate.filename))
~~~

Bản build dùng các phụ thuộc Cargo đã khóa và MSVC CRT tĩnh. Tính năng `python-tests` vẫn tắt. Gói chứa `bin/cirnet-contracts.exe`, tài nguyên schema và catalog runtime chuẩn, cùng `contracts-native-manifest.json` ghi SHA-256 của mã nguồn, revision Git nếu có, trạng thái cây làm việc, toolchain và danh mục SHA-256. Đặt `CIRNET_CONTRACTS_ROOT` thành thư mục `resources` trong gói. Chạy `validate` với thư mục đó và dùng `descriptor` để đọc schema RPC nhúng. Khi thực thi, gói không cần trình biên dịch, Node hoặc Python.

Nếu phiên bản đã ghim và xác minh chỉ được cài dưới bí danh Rustup `stable`, đặt `$env:RUSTUP_TOOLCHAIN = 'stable'` trong shell build. Lệnh đóng gói kiểm tra phiên bản Rust và Cargo thực tế trước khi build.

[Workflow Windows native](../../.github/workflows/native.yml) build ứng viên này và SDK npm. SDK được sinh từ checkout nguồn chuẩn bằng công cụ Node đã khóa trước khi triển khai. Smoke test gói trên máy phát triển tách biệt với các gate cài đặt sạch và an toàn của hệ thống.

Receipt đóng gói ghi nguồn trình biên dịch, SHA-256 của descriptor RPC
và định danh riêng của archive crate Rust. Crate native loại trừ source Python.
Tính năng đối chứng tùy chọn chỉ dùng trong kiểm thử. Mặc định dùng trình
biên dịch từ gói vendored đã khóa. Muốn chọn trình biên dịch thay thế đã
rà soát, truyền `PROTOC` tuyệt đối và `CIRNET_PROTOC_SHA256` chữ thường.
Đối chiếu descriptor sinh ra với baseline trước khi dùng bản dựng thay thế
để tích hợp.

## Sinh và kiểm chứng SDK

Dùng Node 26. Tạo thư mục đầu ra cho kiểm thử SDK trước khi chạy. Mỗi lần chạy `generate-sdk` cần một đường dẫn đích mới, chưa tồn tại, có thư mục cha nằm ngoài kho.

~~~sh
export CIRNET_SDK_NODE_TOOLS="$CIRNET_ARTIFACTS_SESSION/sdk-node-tools"
export CIRNET_SDK_TEST_OUTPUT="$CIRNET_ARTIFACTS_SESSION/sdk-test-output"
mkdir -p "$CIRNET_SDK_NODE_TOOLS" "$CIRNET_SDK_TEST_OUTPUT"
cp packages/public-sdk/package.json packages/public-sdk/package-lock.json "$CIRNET_SDK_NODE_TOOLS/"
npm --prefix "$CIRNET_SDK_NODE_TOOLS" ci --no-audit --no-fund --cache "$CIRNET_ARTIFACTS_SESSION/npm-cache"

cargo test --locked --test generator_parity -- --ignored

sdk_projection="$CIRNET_ARTIFACTS_SESSION/native-sdk"
cargo run --locked --release -- generate-sdk "$repo" "$sdk_projection" "$CIRNET_SDK_NODE_TOOLS"
node packages/public-sdk/scripts/build.mjs --source "$sdk_projection/contracts/browser" \
  --node-tools "$CIRNET_SDK_NODE_TOOLS" --output "$CIRNET_ARTIFACTS_SESSION/native-sdk-dist"
export CIRNET_ARTIFACT_PATHS_MODULE="$(node --input-type=module -e 'import {resolve} from "node:path"; import {pathToFileURL} from "node:url"; console.log(pathToFileURL(resolve("tools/artifact-paths/cirnet_artifact_paths/artifact-paths.mjs")).href)')"
node scripts/check_public_sdk.mjs --source "$sdk_projection/contracts/browser" \
  --node-tools "$CIRNET_SDK_NODE_TOOLS" --output "$CIRNET_ARTIFACTS_SESSION/native-sdk-pack"
~~~

Bộ sinh mặc định đọc `contracts/api/openapi.json` đã chốt. Đối số tùy chọn thứ tư sau `generate-sdk` nhận tệp OpenAPI đã được rà soát. Dùng đúng bản xuất Remote native khi sinh SDK cho profile native. Ghi chung SHA-256 của bản xuất gốc và SDK đóng gói, rồi lưu kết quả kiểm tra của trình sinh và bên tiêu thụ. Tệp `contracts/fingerprints.json` sinh ra mô tả bản chiếu đã chuẩn hóa. Runtime Rust và quy trình SDK này chạy độc lập với Python. Bước kiểm tra gói chạy SDK đã giải nén. Để đối chiếu mô-đun, khai báo kiểu và tài nguyên, dùng `--baseline` với gói cùng phiên bản đã rà soát. So sánh riêng SHA-256 để xác nhận từng byte trùng khớp. [Định danh bản bàn giao và gói](api/compatibility.md#định-danh-phát-hành-và-openapi)

Với bàn giao native đã rà soát, truyền `--native-bundle ABSOLUTE_BUNDLE`,
`--native-bundle-sha256 REVIEWED_MANIFEST_SHA256`, `--native-crate ABSOLUTE_CRATE`
và `--remote-openapi ABSOLUTE_EXPORT` cho `scripts/check_public_sdk.mjs` cùng
`--node-tools` và `--output` như trên. Trình kiểm tra chạy binary đã xác minh
trên crate nguồn đã xác minh rồi ghi cả hai định danh, SHA-256 của OpenAPI gốc
và digest gói npm vào `native-sdk-receipt.json`. Dấu vân tay nguồn `native-v1`
dùng `contracts/capture/layout.json` cho NCAP. Mục manifest đóng băng, bản
chiếu schema và bản chiếu OpenAPI giữ nguyên định danh.

## Fixture đối chứng

| Kiểm thử | Đầu vào và điều kiện kiểm tra |
| --- | --- |
| [Xác thực](../../tests/validation_parity.rs) | [Các ca dùng chung](../../contracts/fixtures/conformance.json), [bài tập](../../contracts/fixtures/assignment-v2-conformance.json), [điều khiển thiết bị đo](../../contracts/fixtures/instrument-control-conformance.json), [lịch sử](../../contracts/fixtures/history-conformance.json) và [fixture thiết kế](../../contracts/design-models/fixtures/) kiểm tra profile, tính tương thích, thiết kế tham chiếu và đầu vào sai. |
| [NCAP](../../tests/capture_parity.rs) | [Các ca NCAP](../../contracts/fixtures/capture/conformance.json) kiểm tra byte NCAP1/NCAP2, thời gian, khoảng trống, đầu vào sai và các mã hóa INT16, UINT16, FLOAT32. |
| [Hardware RPC v2](../../tests/hardware_v2_parity.rs) | [Proto chuẩn](../../contracts/hardware/v2/rpc_v2.proto) là đầu vào của descriptor. Kiểm thử bao quát hai tên dịch vụ, tag trên đường truyền và sự khác nhau giữa trường vắng mặt với trường được gán giá trị mặc định. |
| [Sinh SDK](../../tests/generator_parity.rs) | Lần chạy Cargo mặc định có kiểm tra bản chiếu cục bộ. Chạy riêng năm ca được đánh dấu bỏ qua với bộ công cụ Node đã khóa và thư mục ngoài kho như trên. Các ca này kiểm tra sinh lại khi không có bản chụp trình duyệt đã commit, từ chối đầu vào không an toàn hoặc cũ, và đóng gói. |

## Bằng chứng kiểm chứng

Ghi SHA-256 của mã nguồn, công cụ, kết quả kiểm thử và mã băm artifact trong phiên bên ngoài kho. Ứng viên tích hợp còn ghi SHA-256 chính xác của bản xuất Remote cùng archive SDK. Remote quản lý việc chọn bản phát hành.
