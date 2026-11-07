# Duy trì tài liệu Contracts

[English](../../en/contributing/writing-documentation.md) · [Mục lục](../index.md) · [Chỉ dẫn kho mã](../../../AGENTS.md)

Giữ README gốc bằng tiếng Anh và liên kết đến cả hai mục lục tài liệu. Đặt hướng dẫn hiện hành ở cùng đường dẫn tương đối trong `docs/en/` và `docs/vi/`. Cập nhật hai bản cùng nhau với cùng định danh, ví dụ, giới hạn và trạng thái năng lực. Contracts phụ trách schema, định dạng truyền, danh mục và binding được sinh. Remote phụ trách đăng ký route FastAPI và chọn bản phát hành hệ thống. [Tra cứu hợp đồng](../contract-reference.md)

Dùng [changelog](../../changelogs/README.md) cho thay đổi phiên bản đã xác minh. Giữ một tệp `CHANGELOG_V<major>.md` cho mỗi phiên bản chính đã phát hành, đặt Unreleased trước các bản đã phát hành theo thứ tự giảm dần. Mỗi bản phát hành có tiêu đề SemVer, liên kết trong mục lục, ngày có bằng chứng và commit mà tag trỏ đến. Phân biệt mã nguồn hiện tại, ứng viên đã kiểm chứng và artifact đã công bố khi mô tả trạng thái phát hành. [Bằng chứng kiểm chứng](../native.md#bằng-chứng-kiểm-chứng)

Với mỗi công việc AI, lưu kế hoạch trong `.superpowers/plans/` và đặc tả tương ứng, tiêu chí nghiệm thu, checklist trong `.superpowers/specs/`. Git đã bỏ qua cả hai thư mục. Dùng `- [ ]` cho việc còn mở và `- [x]` sau khi kiểm chứng.

Giữ [hồ sơ kiểm chứng và di chuyển gốc](../history.md) tại đường dẫn đã ghi và bằng ngôn ngữ gốc. Bảo toàn bản OpenAPI đã chốt để bàn giao, các mốc schema và định dạng truyền, fixture và dấu vân tay tài nguyên. Trước khi chuyển hoặc ngừng dùng tài liệu, kiểm tra tham chiếu từ Markdown, gói, quy trình build, CI và kiểm thử, đồng thời xác nhận thay đổi đã được cho phép.

Đối chiếu từng cặp ngôn ngữ liên quan và kiểm tra đường dẫn cục bộ, chữ hoa thường, mốc đề mục và tham chiếu sang kho khác. Giữ bằng chứng thô trong phiên do bộ cấp phát quản lý, bên ngoài checkout, worktree và thư mục Git. Sau khi kiểm chứng, cập nhật cặp tài liệu kỹ thuật và changelog liên quan, rồi kiểm tra Git diff, trạng thái và đầu ra bị bỏ qua. [Hướng dẫn kiểm chứng](../verification.md)
