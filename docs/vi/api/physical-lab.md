# Hợp đồng Physical Lab 1.0

[English](../../en/api/physical-lab.md) · Tiếng Việt

[Mục lục tài liệu](../index.md)

[Schema Lab](../../../contracts/physical-lab/lab.schema.json) định nghĩa Physical Lab 1.0 theo JSON Schema 2020-12. `DetectedTopologyGraph` giữ lỗi nối dây của sinh viên và các quan sát chưa diễn giải được hoặc mơ hồ. Lab 1.0 đã chốt độc lập với CircuitGraph 1.0, 1.1 và bản bổ sung 1.2 của trình biên tập. [Bài tập có phiên bản](../assignment.md) có thể mang đồ thị tham chiếu 1.2 mà không sửa Lab 1.0.

Kết quả an toàn, tô pô và chức năng độc lập. Schema đặt tên riêng cho `SafetyVerificationResult`, `TopologyVerificationResult`, `FunctionalVerificationResult` và `ExecutionDecision`. `InstrumentationExecutionPlan` chỉ định tuyến điểm cuối của phòng thí nghiệm. Dây nối nội bộ của sinh viên vẫn là quan sát, nằm ngoài kế hoạch định tuyến. Tập trường đóng loại trừ điều khiển truyền tải thô. `contracts.validation` kiểm tra ngữ nghĩa về tính duy nhất của điểm cuối và liên kết.

[Ví dụ khả năng v1](../../../contracts/physical-lab/examples/v1-target-capabilities.json) là mục tiêu thiết kế với `available: false`. Giá trị thiết kế, gồm nhãn giao thức minh họa `2.0-scaffold`, không phải bằng chứng runtime. Số kênh, khả năng hỗ trợ và số slot runtime đến từ trình điều khiển và hồ sơ trạm thực tế. Giới hạn điện cần bằng chứng xác nhận.

[Ánh xạ đề xuất FPGA đã ghim](https://github.com/cirnetlab/cirnet-remote/blob/295797c/docs/architecture/fpga-contract-conformance.md) phân biệt `InstrumentationExecutionPlan` đã triển khai với tên lịch sử `PhysicalExecutionPlan` và ảnh cấu hình bo mạch đề xuất. Các điều kiện ABI, độ mới và phần cứng vẫn độc lập. Ánh xạ này không kích hoạt trình điều khiển vật lý. [Tài liệu định dạng NCAP](ncap.md) tách đầu ra runtime v1 khỏi các hàm hỗ trợ lấy mẫu v2. Xem [tài liệu Lab gốc](../../../contracts/physical-lab/README.md).
