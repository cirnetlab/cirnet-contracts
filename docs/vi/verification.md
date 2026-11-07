# Kiểm chứng Contracts và SDK công khai

[English](../en/verification.md) · Tiếng Việt

[Mục lục tài liệu](index.md)

Trang này hướng dẫn kiểm chứng gói Python và SDK. Xem [hướng dẫn kiểm chứng Rust](native.md) cho Cargo, sinh SDK bằng Rust và đối chứng với wheel đã cài.

## Cấp phát phiên

Dùng Node 26 và Git trong PATH. Khi kiểm tra nhiều kho, đặt `CIRNET_ARTIFACTS_CONTEXT` với chủ sở hữu tường minh và mọi gốc Git tham gia trước khi cấp phát. [README bộ cấp phát](../../tools/artifact-paths/README.md) mô tả cách chọn thư mục cha bên ngoài và cách kiểm tra quyền sở hữu.

Chạy các lệnh sau trong một shell POSIX riêng tại thư mục gốc của kho. Shell thoát khi có lỗi. Giữ shell đó để chạy các bước kiểm chứng và dọn phiên.

~~~sh
set -e
CIRNET_ARTIFACTS_SESSION="$(node tools/artifact-paths/artifact-paths.mjs)"
test -n "$CIRNET_ARTIFACTS_SESSION"
export CIRNET_ARTIFACTS_SESSION
printf '%s\n' "$CIRNET_ARTIFACTS_SESSION"
export TMP="$CIRNET_ARTIFACTS_SESSION" TEMP="$CIRNET_ARTIFACTS_SESSION" TMPDIR="$CIRNET_ARTIFACTS_SESSION"
export PIP_CACHE_DIR="$CIRNET_ARTIFACTS_SESSION/pip-cache"
export npm_config_cache="$CIRNET_ARTIFACTS_SESSION/npm-cache"
export PYTHONDONTWRITEBYTECODE=1
~~~

## Kiểm tra các gói Python và SDK

Dùng môi trường Python mới nằm ngoài bản sao kho mã và bộ phụ thuộc dựng và chạy đã khóa mã băm, đã được rà soát. Bộ sinh và kiểm thử mã nguồn dùng phụ thuộc Node đã khóa của SDK.

~~~sh
python -B -m pip download --require-hashes --only-binary=:all: -r requirements-build.lock --dest "$CIRNET_ARTIFACTS_SESSION/build-wheelhouse"
python -B -m pip install --require-hashes --no-index --find-links "$CIRNET_ARTIFACTS_SESSION/build-wheelhouse" -r requirements-build.lock
python -B -m pip install --no-index --no-build-isolation --no-deps ./tools/artifact-paths
npm --prefix packages/public-sdk ci --no-audit --no-fund
python -B -m contracts.codegen.generate_contracts --check
python -B -m contracts.codegen.sync_editor_catalog --check
python -B -m contracts.codegen.sync_breadboard_topologies --check
python -B scripts/check_public_compatibility.py
export CIRNET_ARTIFACT_PATHS_MODULE="$(node --input-type=module -e 'import {resolve} from "node:path"; import {pathToFileURL} from "node:url"; console.log(pathToFileURL(resolve("tools/artifact-paths/cirnet_artifact_paths/artifact-paths.mjs")).href)')"
python -B -m pytest -q --basetemp "$CIRNET_ARTIFACTS_SESSION/pytest"
node scripts/check_public_sdk.mjs --output "$CIRNET_ARTIFACTS_SESSION/sdk-pack"
python -B -m build --no-isolation --wheel --sdist --outdir "$CIRNET_ARTIFACTS_SESSION/dist"
~~~

Bước kiểm tra SDK dựng và giải nén gói npm trong phiên sở hữu, sau đó chạy phần công khai mà không cần phụ thuộc sinh mã. Bước này kiểm tra đủ schema công khai để phân giải tham chiếu và từ chối dữ liệu nội bộ. Bước dựng wheel gọi trình sinh Hardware RPC và đóng gói binding v2.

Để kiểm tra bản đã cài, dùng môi trường mới và thư mục làm việc bên ngoài kho. Cài đầy đủ phụ thuộc `requirements-runtime.lock` ngoại tuyến theo mã băm, rồi cài đúng wheel với `--no-index --no-deps`. Chạy `pip check` và `python -I -B ABSOLUTE_CHECKOUT/scripts/probe_installed_contracts.py`. Dựng độc lập từ sdist đã giải nén và so sánh từng byte của wheel với cùng đầu vào tái lập. Trong Remote, xuất lại OpenAPI với đúng wheel cuối cùng và đối chiếu với bản bàn giao đã chốt.

## Xem kết quả và dọn phiên

Ghi bản sửa đổi đã thử, mã băm gói và phạm vi kiểm chứng. Xem kết quả thô, giữ riêng artifact đã chọn để phát hành và chỉ dọn phiên thuộc quyền sở hữu của mình.

~~~sh
node tools/artifact-paths/artifact-paths.mjs --cleanup "$CIRNET_ARTIFACTS_SESSION"
~~~

[Quy trình độc lập](../../.github/workflows/contracts.yml) ghi đầy đủ các bước dựng và kiểm tra sau cài. [Kết quả theo ngày](../archive/2026-11-05-contracts-verification.md#namespace-verification-2026-11-04) phân biệt lần chạy cục bộ với bằng chứng về artifact đã công bố.
