# Định dạng bản ghi đo NCAP

[English](../../en/api/ncap.md) · Tiếng Việt

[Mục lục tài liệu](../index.md)

[Bố cục nhị phân đã kiểm tra](../../../contracts/capture/layout.json) là nguồn chuẩn về bố cục. Trình sinh hợp đồng tạo hằng số trình duyệt và `contracts.boundary.capture_layout`. Python cũng có thể đọc trực tiếp tài nguyên đóng gói khi sinh mã. [Schema NCAP](../../../contracts/capture/ncap.schema.json) định nghĩa các bộ mô tả mẫu có tập trường đóng tại `https://cirnet.local/contracts/capture/ncap/2.0/schema.json`. Vẫn đọc được CaptureMetadata 1.0 trước đây.

Cả hai phiên bản bắt đầu bằng tiền tố 9 byte. Tiền tố gồm ASCII `NCAP`, một byte phiên bản không dấu và độ dài phần đầu JSON tính theo byte, lưu dưới dạng số nguyên 32 bit không dấu theo thứ tự little-endian. Phần đầu JSON UTF-8 dài 1–65.536 byte, theo sau là đúng phần thân nhị phân. Từ chối UTF-8 sai, BOM, trường trùng kể cả khi giá trị bằng nhau, trường bổ sung và byte thừa cuối phần thân. Trường số nguyên ở phần đầu phải dùng token số nguyên thập phân. Cách viết phân số hoặc số mũ không hợp lệ cho trường đếm nguyên. Số hiệu chuẩn v2 phải giữ các chữ số đã viết sau chuyển đổi thập phân sang binary64 rồi về thập phân. Token số nguyên, gồm cả số nguyên hiệu chuẩn, phải nằm trong miền số nguyên an toàn của JavaScript. Có thể dùng ký pháp khoa học cho giá trị hiệu chuẩn dấu phẩy động lớn biểu diễn được.

## Sự kiện số phiên bản 1

Phần đầu chỉ chứa `encoding: "digital_events"`, bảng `signals` có thứ tự, `sample_count` và `duration_ns`. Mỗi bản ghi little-endian `<HQB` dài 11 byte, gồm chỉ số kênh 16 bit không dấu, số nano giây 64 bit không dấu và một byte giá trị. `0` và `1` là các mức số, còn `2` là mức chưa biết `X`.

Giải mã giữ thứ tự sự kiện, kể cả khi dấu thời gian trùng hoặc không tăng đơn điệu. Giá trị và dấu thời gian giữ đúng mẫu gốc. Dấu thời gian không vượt thời lượng khai báo hoặc 9.007.199.254.740.991. Bảng kênh có tối đa 4.096 tên không rỗng và không trùng nhau. Tải dữ liệu có tối đa 100.000 sự kiện. Trình mã hóa sắp xếp bảng kênh và giữ đúng byte mẫu chuẩn cũ đã lưu.

Điểm cuối runtime, phiên bản MIME và CaptureStore sinh và kiểm tra v1. SHA-256 bao phủ toàn bộ tiền tố, phần đầu JSON và phần thân, được lưu bên ngoài trong CaptureMetadata. Phần đầu nhị phân v1 không có trường checksum.

## Khung mẫu phiên bản 2

`AnalogCaptureDescriptor` 2.0 gồm toàn bộ phần đầu dự kiến, đúng `payload_byte_count` và `sha256` chữ thường của toàn bộ tải nhị phân. Giá trị trong `AnalogCaptureHeader` phải khớp bộ mô tả bất kể thứ tự trường JSON. Định danh và thế hệ bản ghi tham gia cả phép băm lẫn đối chiếu. Mã băm nằm ngoài phần đầu nhị phân để tránh phụ thuộc vòng.

Toàn bộ bản ghi dùng một kiểu mã hóa. `INT16` và `UINT16` dùng hai byte mỗi giá trị. IEEE-754 `FLOAT32` dùng bốn byte. Tất cả theo little-endian. Phần thân xen kẽ các kênh theo thứ tự khai báo trong từng khung. Với hai kênh, phần thân bắt đầu bằng khung 0/kênh 0, khung 0/kênh 1, rồi khung 1/kênh 0. `frame_count` đếm số khung đã lưu. Yêu cầu đúng `frame_count * channels.length * bytes_per_value` byte phần thân. Giới hạn bộ phân tích là 100.000 khung và tám kênh. Giá trị FLOAT32 hoặc kết quả hiệu chuẩn không hữu hạn làm artifact bị từ chối, kể cả trong khoảng được đánh dấu không hợp lệ.

Mỗi kênh khai báo ID, đơn vị, hệ số khác 0, độ lệch, khoảng vật lý tăng dần và ID hiệu chuẩn có thể là null. Công thức chuyển đổi là `raw * scale + offset`. ID hiệu chuẩn null không xác lập khả năng truy xuất hiệu chuẩn. Giá trị ngoài khoảng khai báo được giữ và đánh dấu clipping. Danh sách kênh clipping chỉ được tham chiếu các kênh đã khai báo.

Trục thời gian khai báo `capture_relative`, `unix_epoch` hoặc `device_clock`. Tọa độ tương đối theo bản ghi yêu cầu mốc 0. Tọa độ theo đồng hồ thiết bị yêu cầu ID đồng hồ không diễn giải và không được gắn nhãn UTC. Các tham chiếu còn lại yêu cầu ID đồng hồ null. Trục dùng mốc nano giây dạng chuỗi thập phân và tốc độ mẫu hữu tỉ dương `numerator / denominator` Hz. Thời gian chính xác của khung giữ dạng phân số số nguyên.

Vị trí khoảng mất tăng nghiêm ngặt. Khoảng mất tại chỉ số lưu `i` biểu diễn các khung thiếu trước khung đó. Chỉ số 0 biểu diễn mất đầu bản ghi. Chỉ số `frame_count` biểu diễn mất cuối bản ghi, vẫn là bằng chứng chất lượng dù không có mẫu được lưu. Khung `i` có thời gian chính xác `anchor + (i + sum(gaps at or before i)) * 1e9 * rate.denominator / rate.numerator`.

Chất lượng ghi trạng thái, lý do kết thúc, số khung mất chính xác, khoảng mất, các khoảng khung lưu không hợp lệ đã sắp xếp và không giao nhau, cùng ID kênh clipping. Các khoảng mất giải thích toàn bộ số khung mất. Định dạng cho phép tối đa 256 khoảng mất, 256 khoảng không hợp lệ và 1.000.000 khung mất. Tử số và mẫu số dương của tốc độ mẫu đều có giá trị tối đa 1.000.000.000.000.

| Trạng thái | Ý nghĩa |
| --- | --- |
| `VALID` | Kết thúc hoàn tất, không mất khung và không có khoảng không hợp lệ. |
| `PARTIAL` | Kết thúc chưa hoàn tất hoặc có khoảng không hợp lệ, không mất dữ liệu truyền tải. |
| `LOSS` | Có ghi nhận mất khung. |
| `INVALID` | Giữ bằng chứng và không hiển thị biên độ như dữ liệu tin cậy. |

Các lý do kết thúc là `COMPLETE`, `STOPPED`, `DURATION_LIMIT`, `BUFFER_LIMIT`, `TRANSPORT_LOSS`, `DRIVER_ERROR` và `UNKNOWN`. Chúng không xác nhận thao tác phần cứng đã được áp dụng thành công.

## Giải mã, truyền tải và hiển thị

Python `decode_sampled` kiểm tra cấu trúc bộ mô tả, quan hệ giữa các trường, SHA, sự khớp của phần đầu và bố cục chính xác trước khi trả giá trị thô do kết quả sở hữu. Trình duyệt dùng `decodeAnalogCapture` để giải mã cấu trúc và báo `integrity: "unchecked"`. `verifyAndDecodeAnalogCapture` kiểm tra SHA trước khi báo `integrity: "verified"`. Thiếu hàm kiểm tra toàn vẹn làm thao tác thất bại rõ ràng. Nếu đã hủy, phần thân hoặc mã băm hoàn tất muộn không được công bố.

`loadAnalogCapture` yêu cầu `AnalogPayloadSource` tường minh. Nguồn nhận định danh bản ghi, tín hiệu hủy và giới hạn byte chính xác. Nguồn phải xác minh quyền truy cập của chủ sở hữu và dùng cơ chế truyền tải có phiên bản phù hợp. Runtime vẫn phục vụ v1, Scope mặc định có 0 kênh và chưa có nguồn mạng v2 mặc định. Kết nối dữ liệu mẫu Scope cần hồ sơ trạm tường minh và điểm cuối có phiên bản. Định dạng này chưa có bộ sinh mẫu từ phần cứng.

`createDisplayEnvelope` tạo tối đa 4.096 nhóm cực tiểu/cực đại cho khoảng khung và kênh đã chọn. Hàm giữ các cực trị hợp lệ và giữ nguyên mảng định kiểu thô. Kết quả dùng cho hiển thị mang nguồn, nguồn gốc tổng hợp hay đo thật, toàn vẹn, chất lượng, đơn vị, gián đoạn, vùng không hợp lệ và clipping. Khoảng không hợp lệ bị loại khỏi biên độ. Bản ghi `INVALID` toàn bộ có biên độ null. Thời gian hữu tỉ chính xác vẫn có sẵn bên cạnh độ lệch tương đối gần đúng để vẽ. Hàm không làm mượt, nội suy, nối qua khoảng mất, chấm điểm hoặc thay thế bằng chứng thô.

## Bộ dữ liệu kiểm thử chung

[Bộ dữ liệu kiểm thử tương thích](../../../contracts/fixtures/capture/conformance.json) chứa byte mẫu chuẩn tổng hợp được đóng gói độc lập cho kiểm thử Python và trình duyệt. Bộ này kiểm tra giữ byte cũ, tên UTF-8, dấu thời gian chính xác và mức chưa biết, cả ba kiểu mã hóa mẫu, hiệu chuẩn kể cả hệ số âm, độ chính xác tốc độ và mốc, bản ghi rỗng, một phần, không hợp lệ và bị mất, clipping, phần đầu sai, kích thước phần thân và checksum. Ca kiểm tra ngữ nghĩa có chú thích đối chiếu mã lỗi, đường dẫn, mức độ và profile. Kiểm thử xác nhận khả năng đọc và ghi cùng định dạng. Xem [tài liệu định dạng gốc](../../../contracts/capture/README.md).
