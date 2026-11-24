# Ảnh bộ nhớ cục bộ 1.0

[English](../en/memory.md) · Tiếng Việt

[Mục lục tài liệu](index.md)

[Schema ảnh bộ nhớ](../../contracts/memory/memory-image.schema.json) định nghĩa ảnh byte ban đầu cục bộ của trình biên tập. Mô-đun MEMORY tổng quát lưu ảnh trong `properties.memory`.

~~~json
{"version":"1.0","word_bits":8,"depth":4,"data":[0,1,65,255]}
~~~

Độ sâu từ 1 đến 256, và mỗi byte là số nguyên từ 0 đến 255. Bên tiêu thụ còn kiểm tra ngữ nghĩa `data.length === depth`. Trình duyệt kiểm tra cả hai trước khi đổi đồ thị. Thực thể mới khởi tạo rõ 32 byte bằng 0. Mỗi thực thể sở hữu mảng riêng. Hex Editor nhận byte hai chữ số hệ mười sáu cách nhau bằng khoảng trắng, nạp và lưu ảnh JSON, đồng thời hoàn tác và làm lại qua lịch sử mạch.

`ADDR`, `DATA_IN`, `DATA_OUT`, `WE`, `OE` và `CLK` là các cổng chức năng của trình biên tập. Chúng không xác định chân trên một chip cụ thể. Để mô phỏng thực thi bộ nhớ cần quy tắc địa chỉ, thời gian đọc và ghi, reset và lưu trạng thái. Ảnh cục bộ mô tả cấu hình ban đầu. Đọc RAM vật lý, bản ghi mô phỏng và phần cứng SDRAM nằm ngoài hợp đồng này. Xem [hướng dẫn bộ nhớ gốc](../../contracts/memory/README.md).
