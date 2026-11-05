# Ranh giới điều khiển và giao thức FPGA

[English](../en/fpga-protocol.md) · Tiếng Việt

[Mục lục tài liệu](index.md)

[Schema điều khiển logic](../../contracts/fpga-protocol/control.schema.json) định nghĩa bao lệnh `1.0-scaffold` độc lập với truyền tải, gồm `protocol_version`, `sequence` và `operation`. Hardware Service trên Mini PC phụ trách `FpgaTransport`. Trình duyệt và API tiếp cận phần cứng qua dịch vụ. Xem [kiến trúc Lab-First đã ghim](https://github.com/cirnetlab/cirnet-remote/blob/0d038c7aed15798696223558b4e58f5ffe926aa9/docs/ARCHITECTURE.md).

| Thao tác | ID kiểm thử RTL | Trạng thái cần có hoặc hiệu ứng |
| --- | ---: | --- |
| SAFE_STATE | 0 | Tắt điều khiển đầu ra và định tuyến, giữ FAULT đã chốt. |
| SCAN_BEGIN | 1 | SAFE → SCANNING khi đầu ra đã tắt. |
| PREPARE | 2 | Yêu cầu quét hoàn tất thành công, `safe_condition`, `plan_valid` và thiết lập nhịp khác 0. |
| ROUTE_APPLY | 3 | PREPARED + kế hoạch an toàn hợp lệ + `routing_ready` → ROUTED. |
| POWER_UP | 4 | Yêu cầu ROUTED, `protection_ready`, `watchdog_armed`, `routing_ready`, `safe_condition` và `plan_valid`. |
| ARM | 5 | Yêu cầu POWERED và điều kiện cho phép có bảo vệ còn hiệu lực. |
| START | 6 | Yêu cầu ARMED, điều kiện cho phép có bảo vệ còn hiệu lực và xung nhịp đã chuẩn bị. |
| STOP | 7 | Tắt điều khiển đầu ra và định tuyến, giữ FAULT đã chốt. |
| RESET | 8 | Xóa trạng thái và xung nhịp sau khi hết tín hiệu lỗi. Vô hiệu hóa trình tự trước đó. |
| STATUS | 9 | Xác nhận trạng thái hiện tại mà không bật đầu ra. |

Các ID này thuộc mô phỏng RTL và ranh giới điều khiển logic. Trình tự không hợp lệ đưa bộ điều khiển về SAFE, còn FAULT giữ đến RESET. Các lỗi trừu tượng là OK 0, INVALID_STATE 1, SAFETY_BLOCK 2, SCAN_FAILED 3, INVALID_COMMAND 4 và INTERLOCK_FAULT 5. Hardware RPC v2 có enum lỗi riêng. `scan_success` cho biết đã có quan sát sử dụng được. Tính đúng của tô pô không ngăn quan sát mạch nối dây nếu quan sát đó an toàn. Trình biên dịch gửi phân bổ điểm cuối và thiết bị đo đã được duyệt, không gửi chỉ thị sửa dây nối nội bộ của sinh viên.

## Truyền tải và miền thanh ghi

Đường dự kiến là Mini PC → Hardware Service → FpgaTransport → bộ điều hợp Serial hoặc USB-SPI được chọn. Bộ điều hợp USB3 hoặc Ethernet sau này giữ cùng giới hạn trao đổi và quy tắc an toàn. `SerialFpgaTransport` và `UsbSpiFpgaTransport` hiện báo lỗi `HARDWARE_UNAVAILABLE`, kể cả với yêu cầu đưa về an toàn. Mô phỏng ảo không triển khai bộ giải mã vật lý.

Đóng khung vật lý vẫn là `TBD_BY_HARDWARE_DESIGN`. Cần chọn chính xác cầu nối và thiết bị, datasheet, chân và cực tính reset, thứ tự byte, kích thước khung và thương lượng phiên bản, xử lý phát lại, phát hiện hỏng dữ liệu, thời gian chờ, trạng thái an toàn và watchdog độc lập. Lệnh mất, hỏng hoặc đến muộn phải để đầu ra ở trạng thái tắt.

Các miền thanh ghi dự kiến `1.0-reserved` bao gồm định danh và khả năng, trạng thái an toàn/điều khiển/lỗi/watchdog, trạng thái quét tô pô, định tuyến điểm cuối đã duyệt, trạng thái đầu ra có bảo vệ, cấu hình xung nhịp, Logic Analyzer, Scope 2CH, Generator 2CH, Trigger, Timestamp, SDRAM Arbiter và bắt tay CDC. Địa chỉ, độ rộng, chân và ABI nhị phân chưa được chốt. Khung RTL có tín hiệu bắt tay và điều kiện cho phép đồng bộ. Lỗi và watchdog có thể chặn đầu ra bất đồng bộ và chốt FAULT trong bộ điều khiển trừu tượng. Đồng bộ vật lý, ngắt điện, reset và thời gian cần bằng chứng riêng trên bo mạch.

## Chọn phiên bản

CircuitGraph 1.0/1.1 và bản bổ sung 1.2, Assignment 1.0 và bản bổ sung 2.0, Physical Lab 1.0 và Hardware RPC cục bộ 2.0 có phiên bản độc lập. Bao lệnh logic, miền thanh ghi, khung nhị phân, firmware và hồ sơ phần cứng cũng là các miền riêng. Khi thiếu trình điều khiển vật lý, StationCapabilities vẫn báo giao thức FPGA `UNAVAILABLE`. Chỉ công bố khả năng vật lý sau khi đã triển khai, đo kiểm và xác nhận tương thích.

Xem [bảng lệnh gốc](../../contracts/fpga-protocol/commands.md), [định dạng gói](../../contracts/fpga-protocol/packet-format.md), [bản đồ thanh ghi](../../contracts/fpga-protocol/register-map.md) và [ma trận phiên bản](../../contracts/fpga-protocol/versions.md).
