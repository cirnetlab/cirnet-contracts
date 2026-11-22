# Hợp đồng bài tập có phiên bản

[English](../en/assignment.md) · Tiếng Việt

[Mục lục tài liệu](index.md)

[AssignmentDefinition 1.0](../../contracts/assignment/assignment.schema.json) dùng JSON Schema 2020-12 và chia sẻ định nghĩa với [Physical Lab 1.0](../../contracts/physical-lab/lab.schema.json). Trình kiểm tra ngữ nghĩa trong `contracts.validation` yêu cầu mỗi ID LabEndpoint là duy nhất. `HYBRID` là chế độ được khuyến nghị cho định nghĩa ban đầu. Dịch vụ Python dùng gói Contracts đã cài.

Định nghĩa cũ vẫn dùng được. [AssignmentDefinitionV2 2.0](../../contracts/public/assignments-v2.schema.json) bổ sung khả năng tham chiếu đồ thị CircuitGraph 1.2. Tham chiếu và đánh giá không quyết định an toàn. Phiên bản đồ thị, bài tập, Physical Lab và Hardware RPC độc lập với nhau. Xem [ghi chú bài tập gốc](../../contracts/assignment/README.md) và [lắp đặt và bài tập có phiên bản](../../contracts/design-models/flexible-placement.md).
