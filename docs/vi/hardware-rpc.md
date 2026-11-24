# Hardware RPC 2.0

[English](../en/hardware-rpc.md) · Tiếng Việt

[Mục lục tài liệu](index.md)

[Protobuf v2](../../contracts/hardware/v2/rpc_v2.proto) định nghĩa giao thức vòng đời hiện tại. Tên gói là `cirnet.hardware.v2`. Profile Python đã phát hành còn đăng ký `netcircuit.hardware.v2` và chỉ thử lại tuyến cũ sau khi nhận `UNIMPLEMENTED`. Profile native phục vụ và gọi dịch vụ chuẩn. RPC vòng đời có phiên bản riêng, tách biệt với dịch vụ Simulator Preview riêng giữa Hardware và Remote, cũng như khung worker giữa Hardware và Simulator. Các tệp v1 `experiment.proto`, `instrument.proto` và `station.proto` giữ không gian tên và tương thích mã nguồn ban đầu làm đầu vào lịch sử. Xem [định danh tài nguyên](../../contracts/hardware/README.md) và [bằng chứng đã chốt](../../contracts/compatibility/README.md).

`InstrumentationExecutionPlan` và `ExecutionDecision` phù hợp với Physical Lab 1.0. `StartExecution` nhận định danh trạm và thí nghiệm. `ConfigureVirtualCircuit` chỉ áp dụng cho trạm ảo và bị trạm vật lý từ chối. Dịch vụ dùng enum lỗi ổn định. Bộ xử lý kiểm tra quyền sở hữu, trạng thái và dữ liệu nghiệp vụ. Bên tiêu thụ xác thực kết quả JSON nghiệp vụ. Remote gọi Hardware qua loopback cục bộ hoặc Unix socket trên nền tảng hỗ trợ. Ghi thanh ghi thô nằm ngoài API này.

Bản build native đã khóa biên dịch proto chuẩn và nhúng descriptor chuẩn. Từ checkout, dùng tệp thực thi native để ghi descriptor vào phiên đã cấp phát bên ngoài kho.

~~~sh
cirnet-contracts descriptor > "$CIRNET_ARTIFACTS_SESSION/rpc-v2.pb"
~~~

`ReadCapture` đưa byte NCAP v1 số có giới hạn vào trường 5 `OperationReply.capture_payload`. `result_json` mang trạng thái khả dụng, số lượng và thời lượng, không chứa mẫu. Remote giải mã và đối chiếu dữ liệu nhị phân với bộ mô tả trước khi lưu vào CaptureStore. Bản ghi 100.000 sự kiện vẫn dưới giới hạn nhận gRPC mặc định vì mỗi tên tín hiệu chỉ xuất hiện một lần trong phần đầu có giới hạn. Bản ghi không khả dụng không có dữ liệu nhị phân. Mẫu trình điều khiển sai làm thao tác thất bại rõ ràng. Siêu dữ liệu truyền hoặc dữ liệu nhị phân sai trả `PROTOCOL_MISMATCH`. Dùng hai tiến trình cục bộ dựng từ các binding tương ứng. Bên gửi JSON nội tuyến cũ không tương thích với bên nhận bản ghi nhị phân. Xem [ghi chú RPC gốc](../../contracts/hardware/v2/README.md) và [bố cục NCAP](../../contracts/capture/README.md).
