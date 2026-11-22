# Các mốc tương thích được hỗ trợ

[English](../../en/api/compatibility.md) · Tiếng Việt

[Mục lục tài liệu](../index.md)

## Định danh phát hành và OpenAPI

Hệ thống 0.1.4 đã chọn wheel và SDK 0.1.2 được công bố trong thế hệ kho gốc. Kho này tạo crate Rust và SDK 0.3.0-dev.1. Remote ghi mã nguồn ứng viên, tên tệp, kích thước và SHA-256 cho mỗi artifact được chọn.

| Trường hoặc miền | Ý nghĩa |
| --- | --- |
| `openapi: 3.1.0` | Đặc tả dùng để đọc bản bàn giao. |
| `info.version: 0.1.2` đã chốt | Phiên bản tài liệu bàn giao FastAPI. SHA-256 là `dc39c874386eba2b58f8b5c631cbee0d63ded86cf287aea63f54e4a237ff8f08`. |
| `info.version: 0.2.4` của native | Phiên bản tài liệu do ứng viên Remote 0.2.4 xuất từ bản bàn giao đã chốt. Bản xuất có mã băm và tập thao tác hỗ trợ riêng. |
| `/api/v2` và `operationId` | Đường dẫn chọn biến thể định dạng truyền. ID phân biệt hoa thường, định danh thao tác và binding được sinh. |
| DTO, schema và định dạng truyền | CircuitGraph, bài tập, RPC 2.0, NCAP và bao lệnh FPGA có phiên bản độc lập. |

Hai trường phiên bản OpenAPI có định nghĩa riêng theo [đặc tả OpenAPI 3.1.0](https://spec.openapis.org/oas/v3.1.0.html#info-object). Remote sở hữu các thao tác được xuất. Giữ byte và dấu vân tay đã chốt, gồm siêu dữ liệu. Đối chiếu SDK ứng viên với đúng bản xuất native, rồi ghi SHA-256 của cả bản xuất gốc và SDK đã đóng gói. Dấu vân tay OpenAPI do trình sinh ghi mô tả bản chiếu đã chuẩn hóa. SDK ứng viên sinh từ bản bàn giao FastAPI có phạm vi tương thích khác.

Dùng [SemVer](https://semver.org/) cho crate và SDK npm, cùng quy tắc phiên bản gói Python cho wheel. Trong giai đoạn 0.x, mỗi profile được hỗ trợ giữ ID thao tác đã công bố và phạm vi tương thích đã khai báo cho tới quyết định ngừng hỗ trợ tường minh. Phân loại thay đổi binding hoặc ID là tương thích mã nguồn, thay đổi đường dẫn, DTO, MIME hoặc giao thức là tương thích định dạng truyền, và thay đổi quyền, lỗi hoặc tác dụng là tương thích hành vi. Ghi yêu cầu hoặc quyết định, bên sản xuất và tiêu thụ bị ảnh hưởng, hướng dẫn chuyển đổi, định danh artifact và ca kiểm thử cho phạm vi hỗ trợ. Hậu tố thao tác như `V2` không quy định phiên bản gói.

Khi chuyển đổi, máy khách dùng binding thao tác của profile đã chọn. Nếu chưa rõ kết quả lệnh ghi, đối chiếu trạng thái qua thao tác đọc đã công bố trước khi chủ động gửi lại.

[Mốc mã nguồn](../../../contracts/compatibility/2026-10-16/baseline.json) giữ byte mã nguồn schema và protobuf trước W1 cùng nguồn gốc Git đã ghi. [Mốc định dạng truyền công khai](../../../contracts/compatibility/2026-10-16/public-wire-baseline.json) ghi cấu trúc thao tác và schema công khai sau W7 cùng các mốc bố cục NCAP. Cả hai là mốc được hỗ trợ nhưng chưa phát hành. Chúng không xác nhận ABI đã phát hành hoặc thao tác phần cứng. Bản chụp công khai ghi lại không gian làm việc đã rà soát, không tạo commit hay tag giả cho giai đoạn đó.

## Phạm vi đối chiếu

Trình sinh Python và Rust kiểm tra ID duy nhất trên toàn bộ thao tác HTTP trực tiếp của bản bàn giao trước khi chọn các tuyến SDK. Chúng từ chối cấu trúc thao tác chưa hỗ trợ như webhook, callback và path item qua tham chiếu. Phép so sánh định dạng truyền bảo vệ ID và cấu trúc đã ghi cho các bên tiêu thụ được hỗ trợ.

Mô tả, tiêu đề và ví dụ của schema và thao tác là nội dung biên tập. Trình kiểm tra chỉ đi qua vị trí schema thực sự. Thuộc tính tên `description`, hoặc mặc định hay ví dụ chứa khóa giống schema, vẫn là dữ liệu. Nội dung mô tả quyền sở hữu trong manifest nằm ngoài bản chụp định dạng truyền. Boolean và số vẫn phân biệt. Magic, tiền tố, thứ tự byte, bản ghi, phiên bản và giới hạn tài nguyên NCAP thuộc định dạng được hỗ trợ.

Lệnh kiểm tra không có chế độ cập nhật mốc. Thay đổi phá vỡ tương thích cần phiên bản và cách di chuyển tường minh hoặc quyết định chính sách tương thích đã rà soát. Lưu bản chụp trước đó cùng bằng chứng cho bản thay thế. Chỉ sinh lại dữ liệu không cho phép đổi cấu trúc đã hỗ trợ. CI còn kiểm tra mốc byte schema và proto gốc, bộ dữ liệu Python và trình duyệt dùng chung, ca sửa đổi phải bị từ chối, thao tác được sinh và tài nguyên đã cài. Hành vi bên tiêu thụ, an toàn điện và khả năng vật lý cần kiểm tra riêng.

## Chuyển đổi tên

Tên máy chủ schema và gói RPC v2 hiện tại dùng tên CIRNET chuẩn. Gói native Contracts 0.3.0-dev.1 chỉ xuất descriptor chuẩn. Bí danh URI ngoại tuyến và tuyến RPC cũ thuộc đầu vào đối chứng đã công bố. Phép so sánh định danh mã nguồn chỉ đảo các thay thế chính tả đã duyệt trước khi đối chiếu mã băm gốc. OpenAPI đã chốt giữ siêu dữ liệu cũ cùng bí danh chuẩn. Siêu dữ liệu bí danh không nhất quán làm kiểm tra thất bại. Hai tên dịch vụ kiểm tra trạng thái chính xác `cirnet-api` và `netcircuit-api` được chấp nhận trong bản chụp này. Các giá trị và cấu trúc khác không có ngoại lệ nới rộng.

Bản bàn giao đã chốt đối chiếu `cirnet_session` và `X-CIRNET-CSRF` như bí danh chính xác của `netcircuit_session` và `X-CSRF-Token`. Bản chụp ghi loại, vị trí và yêu cầu thông tin xác thực cùng hai kiểu media bản ghi đo. Profile Python đã kiểm thử cả hai tên cookie, header và Accept. Ứng viên native dùng thông tin xác thực và MIME bản ghi đo chuẩn. [Truyền tải theo profile](ncap.md#truyền-tải-http-theo-profile)

Ba tiêu đề schema đồ thị hiện tại dùng `CIRNET`. Bí danh chính xác theo từng tệp chỉ khôi phục tiêu đề `net*CIRCUIT` ban đầu để đối chiếu mã băm mã nguồn đã chốt. Byte và mã băm mốc được giữ làm bằng chứng gốc. Xem [tài liệu tương thích gốc](../../../contracts/compatibility/README.md).
