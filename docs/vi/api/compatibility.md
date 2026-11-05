# Các mốc tương thích được hỗ trợ

[English](../../en/api/compatibility.md) · Tiếng Việt

[Mục lục tài liệu](../index.md)

[Mốc mã nguồn](../../../contracts/compatibility/2026-10-16/baseline.json) giữ byte mã nguồn schema và protobuf trước W1 cùng nguồn gốc Git đã ghi. [Mốc định dạng truyền công khai](../../../contracts/compatibility/2026-10-16/public-wire-baseline.json) ghi cấu trúc thao tác và schema công khai sau W7 cùng các mốc bố cục NCAP. Cả hai là mốc được hỗ trợ nhưng chưa phát hành. Chúng không xác nhận ABI đã phát hành hoặc thao tác phần cứng. Bản chụp công khai ghi lại không gian làm việc đã rà soát, không tạo commit hay tag giả cho giai đoạn đó.

Sau khi kiểm tra dữ liệu sinh và cấp phát phiên theo [hướng dẫn kiểm chứng](../verification.md), chạy tại thư mục gốc của kho.

~~~sh
python -B scripts/check_public_compatibility.py
python -B -m pytest tests/contracts/test_public_compatibility.py --basetemp "$CIRNET_ARTIFACTS_SESSION/compatibility-pytest"
~~~

## Phạm vi đối chiếu

Trình kiểm tra so sánh từng ID thao tác hiện có, phương thức và đường dẫn, tham số, media và schema yêu cầu và phản hồi, yêu cầu bảo mật toàn cục và thực tế của thao tác, phương ngữ JSON Schema thực tế, định nghĩa thành phần có tên, chú thích URI/phiên bản/profile của schema và các phần bố cục NCAP hiện có. Khi mốc gốc không khai báo bảo mật và phương ngữ ở gốc, dùng mặc định OpenAPI 3.1 là không kế thừa yêu cầu bảo mật và dùng phương ngữ nền OAS 3.1. Cấu trúc hoặc tham chiếu không hỗ trợ làm kiểm tra thất bại rõ ràng. Có thể bổ sung thao tác và hợp đồng có tên hoặc phiên bản riêng cùng các mốc được hỗ trợ. Phép so sánh bao quát cấu trúc truyền đã chốt. Quan hệ kiểu con JSON Schema tổng quát và ngữ nghĩa bên tiêu thụ cần bằng chứng riêng.

Mô tả, tiêu đề và ví dụ của schema và thao tác là nội dung biên tập. Trình kiểm tra chỉ đi qua vị trí schema thực sự. Thuộc tính tên `description`, hoặc mặc định hay ví dụ chứa khóa giống schema, vẫn là dữ liệu. Nội dung mô tả quyền sở hữu trong manifest nằm ngoài bản chụp định dạng truyền. Boolean và số vẫn phân biệt. Magic, tiền tố, thứ tự byte, bản ghi, phiên bản và giới hạn tài nguyên NCAP thuộc định dạng được hỗ trợ.

Lệnh kiểm tra không có chế độ cập nhật mốc. Thay đổi phá vỡ tương thích cần phiên bản và cách di chuyển tường minh hoặc quyết định chính sách tương thích đã rà soát. Giữ phiên bản cũ song song với bản thay thế. Chỉ sinh lại dữ liệu không cho phép đổi cấu trúc đã hỗ trợ. CI còn kiểm tra mốc byte schema và proto gốc, bộ dữ liệu Python và trình duyệt dùng chung, ca sửa đổi phải bị từ chối, thao tác được sinh và tài nguyên đã cài. Hành vi bên tiêu thụ, an toàn điện và khả năng vật lý cần kiểm tra riêng.

## Chuyển đổi tên

Tên máy chủ schema và gói RPC v2 hiện tại dùng tên CIRNET chuẩn. Bí danh URI ngoại tuyến tường minh và hai tuyến RPC giữ hỗ trợ cũ. Phép so sánh định danh mã nguồn chỉ đảo các thay thế chính tả đã duyệt trước khi đối chiếu mã băm gốc. OpenAPI giữ siêu dữ liệu cũ và thêm bí danh chuẩn. Siêu dữ liệu bí danh không nhất quán làm kiểm tra thất bại. Hai tên dịch vụ kiểm tra trạng thái chính xác `cirnet-api` và `netcircuit-api` được chấp nhận cùng nhau. Các giá trị và cấu trúc khác không có ngoại lệ nới rộng.

Tên bảo mật `cirnet_session` và `X-CIRNET-CSRF` là bí danh chính xác của `netcircuit_session` và `X-CSRF-Token`. Vẫn phải giữ loại thông tin xác thực, vị trí, yêu cầu và khả năng runtime chấp nhận tên cũ. Kiểu media bản ghi đo chuẩn được chấp nhận cùng schema media cũ giống hệt. Bỏ media cũ, đổi một trong hai schema hoặc đưa tên xác thực khác làm kiểm tra thất bại. Remote kiểm thử cả cookie, header và Accept của bản ghi đo theo hai tên.

Ba tiêu đề schema đồ thị hiện tại dùng `CIRNET`. Bí danh chính xác theo từng tệp chỉ khôi phục tiêu đề `net*CIRCUIT` ban đầu để đối chiếu mã băm mã nguồn đã chốt. Byte và mã băm mốc được giữ làm bằng chứng gốc. Xem [tài liệu tương thích gốc](../../../contracts/compatibility/README.md).
