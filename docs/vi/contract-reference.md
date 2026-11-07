# Bảng tra hợp đồng dữ liệu

[English](../en/contract-reference.md) · Tiếng Việt

[Mục lục tài liệu](index.md)

[Manifest tài nguyên](../../contracts/manifest.json) ghi định danh, phiên bản, quyền sở hữu và phạm vi truy cập. URI schema dùng `https://cirnet.local/`. Registry ngoại tuyến cũng phân giải URI cũ `https://netcircuit.local/` mà không tải schema qua mạng.

| Chủ đề | Ranh giới hiện tại và tài liệu |
| --- | --- |
| CircuitGraph | Vẫn đọc được 1.0 và 1.1. Bản 1.2 bổ sung cụm lắp linh hoạt và bộ mô tả được ghim. Xem [CircuitGraph](api/circuit-graph.md) và [lắp linh hoạt](../../contracts/design-models/flexible-placement.md). |
| Bài tập và phòng thí nghiệm vật lý | Assignment 1.0 vẫn dùng được song song với AssignmentDefinitionV2 2.0. Physical Lab 1.0 có phiên bản riêng. Xem [bài tập](assignment.md) và [Physical Lab](api/physical-lab.md). |
| API công khai và thời gian thực | Tuyến FastAPI của Remote phụ trách đăng ký thao tác. [Hướng dẫn API](../../contracts/api/README.md) mô tả thao tác cũ và các đường dẫn có phiên bản bổ sung. |
| Bản ghi đo | [Định dạng NCAP](api/ncap.md) mô tả sự kiện số v1 và định dạng lấy mẫu v2. Kiểm tra bố cục và giới hạn độc lập với phiên bản ứng dụng. |
| Trình biên tập và bộ nhớ | Xem [bộ mô tả và thuộc tính trình biên tập](../../contracts/design-models/README.md), [biên soạn danh mục](catalog.md) và [ảnh bộ nhớ](memory.md). |
| Hardware RPC | [RPC 2.0](hardware-rpc.md) dùng `cirnet.hardware.v2`. Binding đăng ký tuyến cũ và chỉ thử lại sau khi nhận `UNIMPLEMENTED`. Bước dựng wheel sinh binding v2. |
| Bao lệnh FPGA | [Hướng dẫn điều khiển và giao thức](fpga-protocol.md) mô tả bao logic `1.0-scaffold` và bản đồ dự kiến `1.0-reserved`. Khung truyền tải và firmware vật lý cần bằng chứng phần cứng. |
| Tương thích | [Các mốc đã chốt](api/compatibility.md) bao quát định dạng truyền công khai, schema và cấu trúc NCAP được hỗ trợ. Dùng hợp đồng có phiên bản riêng cho thay đổi phá vỡ tương thích. |

Phiên bản CircuitGraph, Assignment, Lab, RPC, NCAP và FPGA độc lập với phiên bản bản phân phối. Kiểm tra schema hoặc giao tiếp không xác nhận an toàn điện hoặc khả dụng vật lý.

## Triển khai Rust

Crate `cirnet-contracts-native` phân giải ngoại tuyến cả hai host URI của schema từ thư mục nguồn được chỉ định và cung cấp descriptor Hardware RPC chuẩn lẫn cũ. Module NCAP trả thời điểm frame lấy mẫu dưới dạng tử số thập phân chính xác và mẫu số nguyên. Chuỗi đầu vào dùng các giá trị vô hướng Unicode. [Hướng dẫn build, CLI và đối chứng](native.md)

## Nguồn quản lý tài nguyên

- [Device Library](../../device-library/README.md) là nguồn danh mục có thể sửa. Quá trình sinh tạo bản sao `contracts/catalog-source/` trong gói cùng các bản chiếu danh mục. Wheel đã cài đọc bản sao đó.
- [Tài nguyên SDK công khai](../../packages/public-sdk/README.md) gồm trình xác thực, thao tác và fixture công khai. `contracts/browser/artifacts.json` ánh xạ đường dẫn bản chụp cũ đến tài nguyên đã cài do `contracts.resources.generated_artifacts()` cung cấp. Consumer lấy artifact này từ Contracts.
- [Bản OpenAPI Remote bàn giao](../../contracts/api/README.md) lấy từ các thao tác đã đăng ký trong FastAPI. `contracts/fingerprints.json` ghi riêng dấu vân tay của nguồn chuẩn, schema công khai và OpenAPI. Manifest phát hành lưu mã băm của các gói phân phối.
- [Quyền sở hữu kiểm thử](../../migration/test-ownership.json) phân biệt kiểm thử hợp đồng độc lập trong `tests/contracts/` với kiểm thử của consumer và workflow Remote được giữ lại trong `migration/consumer-tests/`.
