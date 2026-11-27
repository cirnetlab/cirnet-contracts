# Bảng tra hợp đồng dữ liệu

[English](../en/contract-reference.md) · Tiếng Việt

[Mục lục tài liệu](index.md)

[Manifest tài nguyên](../../contracts/manifest.json) ghi định danh, phiên bản, quyền sở hữu và phạm vi truy cập. URI schema dùng `https://cirnet.local/`. Registry ngoại tuyến cũng phân giải URI cũ `https://netcircuit.local/` mà không tải schema qua mạng.

[Kiến trúc Ecosystem](https://github.com/cirnetlab/cirnet-rules/blob/master/docs/tai-lieu-kien-tuc-he-thong/cirnetlab-architecture-vi.md) phân công quyền sở hữu hợp đồng dùng chung. [Kiến trúc Lab-First](https://github.com/cirnetlab/cirnet-rules/blob/master/docs/tai-lieu-kien-tuc-he-thong/cirnet-architecture-lab-first-vi.md) quy định cách các dịch vụ Lab sử dụng các hợp đồng đó.

| Bên sở hữu | Ranh giới |
| --- | --- |
| Contracts | Schema chuẩn của CircuitGraph và các dữ liệu khác, đầu vào danh mục thiết bị, descriptor Hardware RPC và giao thức logic FPGA, định dạng NCAP, fixture kiểm tra tuân thủ, binding Rust và binding TypeScript sinh tự động. |
| Remote | Đăng ký tuyến, hành vi và phân quyền ứng dụng, bản xuất OpenAPI của từng profile API và lựa chọn bản phát hành hệ thống. Contracts dùng bản xuất đã được rà soát để sinh SDK. |
| Hardware và FPGA | Hardware sở hữu hành vi trạm cục bộ và lớp truyền tải nội bộ. FPGA sở hữu RTL bộ điều khiển và bằng chứng trên bo. Contracts sở hữu định dạng logic dùng chung tại các ranh giới đó. |

| Chủ đề | Ranh giới hiện tại và tài liệu |
| --- | --- |
| CircuitGraph | Đồ thị là nguồn dữ liệu chuẩn của thiết kế logic. Hình học hiển thị trong 3D không tạo kết nối điện. Vẫn đọc được phiên bản 1.0 và 1.1. Bản 1.2 bổ sung cụm lắp linh hoạt và bộ mô tả được ghim. Xem [CircuitGraph](api/circuit-graph.md) và [lắp linh hoạt](../../contracts/design-models/flexible-placement.md). |
| Bài tập và phòng thí nghiệm vật lý | Assignment 1.0 vẫn dùng được song song với AssignmentDefinitionV2 2.0. Physical Lab 1.0 có phiên bản riêng. Xem [bài tập](assignment.md) và [Physical Lab](api/physical-lab.md). |
| API công khai và thời gian thực | Remote phụ trách đăng ký thao tác. Bản bàn giao FastAPI đã chốt lưu API Python. Ứng viên native xuất các thao tác được hỗ trợ để dựng SDK riêng. Xem [định danh phát hành và OpenAPI](api/compatibility.md#định-danh-phát-hành-và-openapi). |
| Bản ghi đo | [Định dạng NCAP](api/ncap.md) mô tả sự kiện số v1 và định dạng lấy mẫu v2. Kiểm tra bố cục và giới hạn độc lập với phiên bản ứng dụng. |
| Trình biên tập và bộ nhớ | Xem [bộ mô tả và thuộc tính trình biên tập](../../contracts/design-models/README.md), [biên soạn danh mục](catalog.md) và [ảnh bộ nhớ](memory.md). |
| Hardware RPC | [RPC 2.0](hardware-rpc.md) dùng `cirnet.hardware.v2`. Profile Python đã phát hành hỗ trợ tên dịch vụ cũ. Profile native dùng dịch vụ chuẩn. |
| Bao lệnh FPGA | [Hướng dẫn điều khiển và giao thức](fpga-protocol.md) mô tả bao logic `1.0-scaffold` và bản đồ dự kiến `1.0-reserved`. Khung truyền tải và firmware vật lý cần bằng chứng phần cứng. |
| Tương thích | [Các mốc đã chốt](api/compatibility.md) bao quát định dạng truyền công khai, schema và cấu trúc NCAP được hỗ trợ. Dùng hợp đồng có phiên bản riêng cho thay đổi phá vỡ tương thích. |

Phiên bản CircuitGraph, Assignment, Lab, RPC, NCAP và FPGA độc lập với phiên bản bản phân phối. Kiểm tra schema hoặc giao tiếp không xác nhận an toàn điện hoặc khả dụng vật lý.

## Triển khai Rust

Crate `cirnet-contracts-native` phân giải ngoại tuyến cả hai host URI của schema từ thư mục nguồn được chỉ định. Bản build 0.3.0-dev.1 xuất descriptor Hardware RPC chuẩn. Module NCAP trả thời điểm frame lấy mẫu dưới dạng tử số thập phân chính xác và mẫu số nguyên. Chuỗi đầu vào dùng các giá trị vô hướng Unicode. [Hướng dẫn build, CLI và đối chứng](native.md)

## Đường dẫn trong kho

| Đường dẫn | Trách nhiệm |
| --- | --- |
| [`contracts/`](../../contracts/README.md) | Các nhóm schema chuẩn, profile xác thực, protobuf Hardware RPC và bố cục NCAP. Manifest tài nguyên hiện hành đăng ký đường dẫn nguồn và bản chiếu. Bản OpenAPI lưu trong kho bảo toàn profile FastAPI đã phát hành. Remote cung cấp đúng bản xuất cho mỗi lần build SDK native. |
| [`device-library/`](../../device-library/README.md) và [`contracts/design-models/`](../../contracts/design-models/README.md) | Sửa định nghĩa thiết bị trong Device Library, còn cấu trúc liên kết và quy tắc cơ khí dùng chung nằm trong design models. Bộ sinh native chiếu dữ liệu linh kiện vật lý và cổng tín hiệu vào các đường dẫn danh mục runtime dưới `contracts/design-models/` và xuất danh mục trình duyệt ở `contracts/browser/data/`. [Bản sao catalog-source](../../contracts/catalog-source/README.md) ghi lại cấu trúc gói Python trước đây. |
| `src/` và `build.rs` | Xác thực Rust, codec NCAP, bộ sinh SDK và CLI. Bản build Cargo biên dịch proto RPC chuẩn thành kiểu Prost và descriptor nhúng trong thư mục đầu ra của Cargo. |
| `contracts/browser/` và [`packages/public-sdk/`](../../packages/public-sdk/README.md) | Các bản chiếu trình duyệt lưu trong kho là đầu vào đối chiếu. Quy trình sinh native ghi bản chiếu mới từ nguồn chuẩn và bản xuất OpenAPI được chọn. Gói npm được build từ bản chiếu đó. `contracts/browser/artifacts.json` ghi lại ánh xạ cũ giữa đường dẫn sao chép và tài nguyên gói Python. |
| `contracts/codegen/`, `scripts/` và [`tools/artifact-paths/`](../../tools/artifact-paths/README.md) | Bộ sinh Node tạo TypeScript và trình xác thực biên dịch sẵn từ các bộ schema do Rust chuẩn bị. Các script build và kiểm tra gói native và npm. Công cụ cấp phát là gói phát triển riêng để lưu đầu ra bên ngoài kho. [Lệnh build và kiểm chứng](native.md) |
| `contracts/fixtures/`, `contracts/design-models/fixtures/` và [`tests/fixtures/`](../../tests/fixtures/README.md) | Dữ liệu kiểm tra tuân thủ hợp đồng cung cấp đầu vào cho kiểm thử native và các fixture SDK được chọn. Thư mục fixture kiểm thử lưu kết quả chuẩn để đối chiếu và bằng chứng trình duyệt trước đây. Các mốc schema và định dạng truyền đã chốt nằm trong [`contracts/compatibility/`](../../contracts/compatibility/README.md). |

Tệp `contracts/fingerprints.json` trong kho lưu bản ghi dấu vân tay trước đây. Mỗi lần sinh native ghi dấu vân tay riêng của nguồn chuẩn, schema công khai và OpenAPI đã chuẩn hóa vào cây đầu ra. Manifest phát hành ghi mã băm của các gói phân phối. [Định danh phát hành và OpenAPI](api/compatibility.md#định-danh-phát-hành-và-openapi)
