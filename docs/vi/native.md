# Build và kiểm chứng Contracts Rust

[English](../en/native.md) · Tiếng Việt · [Mục lục tài liệu](index.md)

Crate `cirnet-contracts-native` 0.1.2 cung cấp trình xác thực ngoại tuyến, codec NCAP1 và NCAP2, kiểu dữ liệu và descriptor Hardware RPC, cùng bộ sinh SDK TypeScript. Tệp thực thi có tên `cirnet-contracts`. Manifest, profile schema, thư viện thiết bị, bố cục capture, proto RPC và OpenAPI đã chốt là đầu vào dùng chung. [Tra cứu hợp đồng dữ liệu](contract-reference.md)

## Build và chạy

Dùng [Rust 1.90.0](../../rust-toolchain.toml) và [Cargo.lock](../../Cargo.lock). Gán `CIRNET_ARTIFACTS_SESSION` bằng phiên ngoài kho do bộ cấp phát quản lý rồi chạy các lệnh POSIX sau tại thư mục gốc của kho mã. Giữ nguyên shell để chạy các bước tiếp theo. [Cấp phát phiên](verification.md#cấp-phát-phiên) và [dọn phiên sau khi xem kết quả](verification.md#xem-kết-quả-và-dọn-phiên).

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
cargo run --locked --release -- descriptor legacy > "$CIRNET_ARTIFACTS_SESSION/rpc-v2-legacy.pb"
~~~

`validate` nhận đường dẫn gốc của mã nguồn, tên hợp đồng và profile. Lệnh đọc JSON từ đầu vào chuẩn, ghi JSON đã kiểm tra ra đầu ra chuẩn và trả mã thoát 2 kèm lỗi có cấu trúc trên luồng lỗi chuẩn khi dữ liệu vi phạm hợp đồng. Các lệnh descriptor ghi descriptor protobuf dạng nhị phân.

## Sinh và kiểm chứng SDK

Dùng Node 26. Gán `selected_sdk` bằng đường dẫn tuyệt đối của gói npm đã chọn sau khi đối chiếu SHA-256 với manifest hệ thống. Tạo thư mục đầu ra cho kiểm thử SDK trước khi chạy. Mỗi lần chạy `generate-sdk` cần một đường dẫn đích mới, chưa tồn tại, có thư mục cha nằm ngoài kho.

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
  --node-tools "$CIRNET_SDK_NODE_TOOLS" --output "$CIRNET_ARTIFACTS_SESSION/native-sdk-pack" \
  --baseline "$selected_sdk"
~~~

Bộ sinh mặc định đọc `contracts/api/openapi.json`. Đối số tùy chọn thứ tư sau `generate-sdk` nhận tệp OpenAPI đã được rà soát. Runtime Rust và quy trình SDK này chạy độc lập với Python. Bước kiểm tra gói chạy SDK đã giải nén, sau đó đối chiếu mô-đun, khai báo kiểu và tài nguyên với gói đã chọn. So sánh riêng SHA-256 của gói để xác nhận từng byte trùng khớp.

## Fixture đối chứng

| Kiểm thử | Đầu vào và điều kiện kiểm tra |
| --- | --- |
| [Xác thực](../../native/tests/validation_parity.rs) | [Các ca dùng chung](../../contracts/fixtures/conformance.json), [bài tập](../../contracts/fixtures/assignment-v2-conformance.json), [điều khiển thiết bị đo](../../contracts/fixtures/instrument-control-conformance.json), [lịch sử](../../contracts/fixtures/history-conformance.json) và [fixture thiết kế](../../contracts/design-models/fixtures/) kiểm tra profile, tính tương thích, thiết kế tham chiếu và đầu vào sai. |
| [NCAP](../../native/tests/capture_parity.rs) | [Các ca NCAP](../../contracts/fixtures/capture/conformance.json) kiểm tra byte NCAP1/NCAP2, thời gian, khoảng trống, đầu vào sai và các mã hóa INT16, UINT16, FLOAT32. |
| [Hardware RPC v2](../../native/tests/hardware_v2_parity.rs) | [Proto chuẩn](../../contracts/hardware/v2/rpc_v2.proto) là đầu vào của descriptor. Kiểm thử bao quát hai tên dịch vụ, tag trên đường truyền và sự khác nhau giữa trường vắng mặt với trường được gán giá trị mặc định. |
| [Sinh SDK](../../native/tests/generator_parity.rs) | Lần chạy Cargo mặc định có kiểm tra bản chiếu cục bộ. Chạy riêng năm ca được đánh dấu bỏ qua với bộ công cụ Node đã khóa và thư mục ngoài kho như trên. Các ca này kiểm tra sinh lại khi không có bản chụp trình duyệt đã commit, từ chối đầu vào không an toàn hoặc cũ, và đóng gói. |

## Đối chứng tùy chọn với wheel đã cài

Chuẩn bị môi trường Python riêng ngoài kho với pytest, toàn bộ phụ thuộc runtime đã khóa gồm phụ thuộc RPC, và wheel 0.1.2 đã chọn được cài trực tiếp từ tệp đã đối chiếu mã băm. Gán `venv_python` bằng đường dẫn tuyệt đối của Python trong môi trường đó. Kiểm thử đối chứng xác nhận mã băm wheel trong metadata của gói đã cài và nạp extension do Cargo dựng từ đường dẫn tuyệt đối được cung cấp. Binding phục vụ kiểm thử.

~~~sh
PYO3_PYTHON="$venv_python" cargo build --locked --release --features python-tests
export CIRNET_CONTRACTS_PYTHON_SHA256=7b092af5817b0e15f02f9f5f292a8e1b20fe1d512600f9c0260d05a58ddaaf42
export CIRNET_CONTRACTS_NATIVE_BINDING="$CARGO_TARGET_DIR/release/libcirnet_contracts_native.so"
(
  cd "$CIRNET_ARTIFACTS_SESSION"
  "$venv_python" -I -B -m pytest -q -p no:cacheprovider \
    --basetemp "$CIRNET_ARTIFACTS_SESSION/native-pytest" \
    "$repo/native/tests/test_installed_native_parity.py"
)
~~~

Ví dụ dùng tên extension trên Linux. Trên Windows MSVC, sao chép `cirnet_contracts_native.dll` từ đầu ra release sang tệp `cirnet_contracts_native.pyd` ngoài kho rồi trỏ `CIRNET_CONTRACTS_NATIVE_BINDING` vào bản sao trước khi chạy pytest. Dùng cùng một môi trường Python cho bước dựng PyO3 và bước đối chứng.

## Bằng chứng kiểm chứng

- **Mã nguồn hiện tại.** Commit triển khai Rust `e640f80b1199779659d3db7817fee16d13cc24c1` dựa trên `f559ec5b21ea7ecf50db4dc5c5e1dc7c5eb5ad39`. Khi ghi kết quả mới, dùng đúng commit của bản mã đã rà soát.
- **Ứng viên đã kiểm chứng.** Gate 2 đạt trên R2 với snapshot nguồn có SHA-256 `d9d6de46c7cc4b46cf2fe0926ce962a5f2775a2bb8965a0e74e26f2a8447aca8`, gồm 42 ca Rust trên Windows, 146 ca đối chứng wheel đã cài, 35 ca runtime native trên Linux và đối chứng SDK đã khóa với Interface. Hồ sơ giữ kết quả chưa đạt của ứng viên đầu tiên khi chẩn đoán dữ liệu vi phạm nhiều ràng buộc. Các đầu vào khác vi phạm đồng thời nhiều ràng buộc schema vẫn có thể cho lỗi đầu tiên khác nhau. [Bằng chứng ứng viên giai đoạn 2 và R2](https://github.com/cirnetlab/cirnet-remote/blob/13a35145b6d15644c7be10d95a5410bf69e11867/docs/vi/architecture/native-migration.md#xác-minh-ứng-viên-giai-đoạn-2)
- **Bản phát hành đã chọn.** Manifest hệ thống 0.1.4 của Remote chọn wheel Python và SDK npm của Contracts v0.1.2 từ `524cc7a5e2e65f39077feefe70c7fb2ae3a773d6`. Gói SDK R2 trùng với gói npm đã chọn. Tệp thực thi và crate R2 là artifact của ứng viên. [Manifest phát hành hệ thống](https://github.com/cirnetlab/cirnet-remote/blob/master/integration/release-set.json)
- **CI và bản sửa đổi tài liệu này.** CI độc lập chạy quy trình Python và SDK hiện hành trên Linux với Python 3.11/3.14 và Node 26. Cargo, các ca SDK Rust chạy ngoài kho và đối chứng PyO3 nằm ngoài workflow này. Build Rust, kiểm thử đối chứng, CI trên GitHub và công bố bản phát hành đều **NOT RUN** trong bản sửa đổi tài liệu này. [Workflow độc lập](../../.github/workflows/contracts.yml)
