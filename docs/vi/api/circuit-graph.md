# Hợp đồng Circuit Graph

[English](../../en/api/circuit-graph.md) · Tiếng Việt

[Mục lục tài liệu](../index.md)

[CircuitGraph](../../../contracts/circuit-schema/circuit.schema.json) biểu diễn ý định logic ở cấp ứng dụng. Cấu trúc 1.0 yêu cầu `schema_version`, `circuit_id`, `modules` và `connections`. Đây là ví dụ tối thiểu về cấu trúc.

~~~json
{"schema_version":"1.0","circuit_id":"empty","modules":[],"connections":[]}
~~~

Không đưa địa chỉ thanh ghi FPGA, đường dẫn SPI Linux hoặc địa chỉ ma trận chuyển mạch thô vào đồ thị. Hardware Service phụ trách chuyển đổi sang thao tác phần cứng. Thay đổi phá vỡ tương thích cần phiên bản schema mới và cập nhật bên tiêu thụ.

Cùng schema đọc được lắp đặt vật lý cứng 1.1 qua [schema mô-đun](../../../contracts/circuit-schema/module.schema.json). [Schema 1.2](../../../contracts/circuit-schema/circuit-v1.2.schema.json) bổ sung cụm lắp linh hoạt và bộ mô tả vật lý tự do được ghim. `AnySupportedCircuitGraph` chọn cấu trúc cũ hoặc 1.2 được hỗ trợ. Trình đọc 1.0 và 1.1 cũ giữ ranh giới phiên bản đã chốt. Xem [bố trí linh kiện](https://github.com/cirnetlab/cirnet-remote/blob/295797c/docs/COMPONENT_PLACEMENT.md) và [lắp đặt linh hoạt](../../../contracts/design-models/flexible-placement.md) để biết hành vi lưu trữ và API.

## Trường trình biên tập cũ

Mô-đun có thể chứa `position: {x,y,z}` trong hệ tọa độ thế giới Three, với Y là độ cao và XZ là mặt làm việc. `rotation` không bắt buộc, tính theo độ quanh trục Y của thế giới. Các trường trình bày này không đổi ý nghĩa điện. Schema mô-đun có thể mở rộng vốn đã cho phép xoay. Trường này được định nghĩa rõ kiểu số mà không đổi phiên bản đồ thị 1.0.

Một [kết nối](../../../contracts/circuit-schema/connection.schema.json) có các điểm cuối được đặt tên.

~~~json
{"source":"moduleID.portID","destination":"moduleID.portID"}
~~~

Lệnh biên tập dùng cổng từ [danh mục thiết bị](../../../device-library/editor/components.json). Lệnh từ chối hướng hoặc độ rộng không tương thích, cạnh trùng và bộ điều khiển đầu vào bổ sung. Cho phép một đầu ra nối nhiều đầu vào. Đầu cuối thụ động dùng `inout`. Backend và bộ mô phỏng phân tích mạng điện. Xóa mô-đun xóa các cạnh nối với nó, còn di chuyển hoặc xoay giữ nguyên ID điểm cuối.

Nhập cục bộ từ chối ID mô-đun trùng, tham chiếu tới mô-đun không tồn tại, hình học không hữu hạn và ảnh bộ nhớ cục bộ sai. Kiểu linh kiện và tên cổng chưa nhận diện vẫn được giữ khi nhập. Hình biểu diễn dự phòng và Inspector hiển thị chúng mà không tự tạo ánh xạ. Schema hoặc API chấp nhận cấu trúc chưa xác nhận tính đúng điện hoặc khả năng thực thi. Xem [tài liệu đồ thị gốc](../../../contracts/circuit-schema/README.md) và [ví dụ lắp cứng](../../../contracts/circuit-schema/examples/physical-hc08-led.json).
