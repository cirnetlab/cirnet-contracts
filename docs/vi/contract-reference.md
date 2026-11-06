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

## Ứng viên Rust Contracts

Crate `cirnet-contracts-native` đọc manifest chuẩn và profile schema từ
thư mục nguồn được chỉ định. Crate phân giải ngoại tuyến cả hai host URI
của schema, sinh kiểu Rust cho Hardware RPC và cung cấp descriptor chuẩn
cùng descriptor cũ. Module NCAP xử lý capture số và capture lấy mẫu,
trả thời điểm frame dưới dạng tử số thập phân chính xác và mẫu số nguyên.
Chuỗi đầu vào dùng các giá trị vô hướng Unicode.

Dựng bằng `cargo build --locked --release`. Toolchain được ghim trong
`rust-toolchain.toml`. Lệnh `cirnet-contracts validate` đọc JSON từ đầu
vào chuẩn và in kết quả đã kiểm hoặc lỗi có cấu trúc. Dùng
`cirnet-contracts descriptor` hoặc `cirnet-contracts descriptor legacy`
để đọc descriptor nhị phân tương ứng.

Cài công cụ Node từ lockfile SDK trong thư mục ngoài repo có chủ sở hữu.
Chạy `cirnet-contracts generate-sdk <source-root> <output-root> <node-tools>`
để sinh dữ liệu từ nguồn chuẩn. Truyền thư mục `contracts/browser` đã
sinh qua `--source` và thư mục công cụ đã khóa qua `--node-tools` cho
bước dựng SDK. Quá trình dựng Rust và sinh SDK chạy được khi không có
Python. Tính năng Cargo tùy chọn `python-tests` dựng binding để đối
chứng với wheel cũ. Consumer chuyển tiếp tiếp tục dùng wheel Python 0.1.2.
