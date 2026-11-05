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
