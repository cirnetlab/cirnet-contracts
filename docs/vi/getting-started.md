# Bắt đầu với CIRNET Contracts

[English](../en/getting-started.md) · Tiếng Việt

[Mục lục tài liệu](index.md)

Mã nguồn hiện tại dựng gói Python `cirnet-contracts==0.1.2` và gói trình duyệt `@cirnet/contracts@0.1.2`. Tên import Python là `contracts`, yêu cầu Python từ 3.11. Dùng artifact và mã SHA256 từ danh mục phát hành hệ thống đã chọn. Dùng môi trường riêng cho các wheel `netcircuit-contracts` cũ vì cả hai bản phân phối đều sở hữu cùng tên import.

Sau khi cài wheel Python đã chọn, chạy kiểm tra phiên bản bản phân phối từ ngoài bản sao kho mã.

~~~sh
python -I -B -c "from importlib.metadata import version; print(version('cirnet-contracts'))"
~~~

SDK trình duyệt cung cấp DTO công khai, trình xác thực ngoại tuyến, siêu dữ liệu thao tác và bố cục bản ghi đo, cùng các bản chiếu danh mục công khai. Gói gồm tập schema công khai cần để giải mọi tham chiếu và bộ dữ liệu kiểm thử có tên. Protobuf RPC, nguồn danh mục nội bộ và mốc tương thích thuộc tài nguyên phía Python. Xem [README gói SDK](../../packages/public-sdk/README.md).

Bộ cấp phát DEV độc lập là `cirnet-artifact-paths` cho Python và `@cirnet/artifact-paths` cho Node. Công cụ này quản lý vị trí đầu ra tạm thời và dọn phiên có kiểm tra quyền sở hữu. Cài công cụ riêng với Contracts runtime. Thiết lập mới dùng `CIRNET_*` và được ưu tiên hơn `NETCIRCUIT_*` cũ, kể cả khi giá trị mới rỗng. Xem [README bộ cấp phát](../../tools/artifact-paths/README.md).

Contracts phụ trách cấu trúc schema, registry ngoại tuyến và binding được sinh. Remote phụ trách đăng ký thao tác FastAPI và chọn bản phát hành hệ thống. OpenAPI được chốt để bàn giao phải trùng bản xuất từ Remote chạy trên đúng wheel cuối cùng. Xem [bảng tra hợp đồng](contract-reference.md) và [hướng dẫn kiểm chứng](verification.md).
