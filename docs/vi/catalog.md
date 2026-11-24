# Biên soạn danh mục

[English](../en/catalog.md) · Tiếng Việt

[Mục lục tài liệu](index.md)

Sửa định nghĩa thiết bị trong [device-library](../../device-library/README.md). [Danh mục linh kiện vật lý](../../device-library/parts/README.md) mô tả phiên bản linh kiện, kiểu vỏ và footprint, định danh chân và nguồn gốc. Nguồn JSON có thể sửa tách `parts` hiện tại khỏi `historicalParts` bất biến. Tăng phiên bản liên quan khi thay đổi sơ đồ chân hoặc ý nghĩa cơ khí hay điện.

Sau khi rà soát thay đổi nguồn, sinh lại dữ liệu từ thư mục gốc của kho Contracts.

~~~sh
cirnet-contracts generate-sdk "$PWD" "$CIRNET_ARTIFACTS_SESSION/catalog-projection" "$CIRNET_SDK_NODE_TOOLS"
~~~

Bộ sinh native đọc định nghĩa chuẩn và ghi danh mục trình biên tập cùng mô tả biên soạn vào thư mục contracts/browser/data của bản chiếu ngoài kho. Xem [hướng dẫn native](native.md) để chuẩn bị bộ công cụ đã khóa.

Phân biệt định danh chân vật lý với tên cổng logic và lưới 3D. Linh kiện có thể hỗ trợ biên soạn và lắp đặt trước khi có hồ sơ thực thi. Tiếp điểm và thanh dẫn của breadboard được xác định từ tô pô khai báo. Thanh dẫn không tự có điện áp. Giữ liên kết tới đúng phiên bản linh kiện và kiểu vỏ đã lưu, đồng thời dựng lại tường minh khi muốn dùng phiên bản mới hơn. Xem [lắp linh hoạt](../../contracts/design-models/flexible-placement.md).
