# Omni Chat (The Cowork Space) PRD

## Problem Statement
Người dùng cần một không gian nghiên cứu, phân tích và tìm kiếm thông tin chuyên sâu (Deep Research) trong IDE mà không bị giới hạn bởi không gian chỉ phục vụ code. Hiện tại, Assistant Panel chỉ tập trung vào việc chat về code hiện tại, không hỗ trợ tốt cho việc brainstorm, lấy context từ web hay tạo các spec.

## Solution
Tạo ra một không gian hội thoại độc lập (Omni Chat) lấy cảm hứng từ Claude Cowork. Không gian này sẽ hỗ trợ việc nghiên cứu sâu, tích hợp công cụ tìm kiếm và cào dữ liệu từ web, có khả năng kết nối qua Omni Server để tìm các skill chuyên môn. Sau khi quá trình brainstorm kết thúc, người dùng có thể "Transfer Context" sang không gian thiết kế hoặc lập trình mà không cần lặp lại ngữ cảnh.

## User Stories
1. As a developer, I want to chat with an AI in a dedicated research panel, so that I can brainstorm ideas without cluttering my code editor.
2. As a researcher, I want to use the `/omni-server` command within the chat, so that I can automatically load relevant research skills to assist me.
3. As a developer, I want to see the list of context sources (URLs, PDFs) the AI has indexed on the right panel, so that I know what data the AI is basing its answers on.
4. As a product manager, I want to click a button to export the current conversation into a Markdown PRD, so that I can formalize the requirements quickly.
5. As a developer, I want to "Transfer Context" to Omni Design or Omni IDE, so that the AI in the next phase has full access to the research context.

## Technical & Architecture Decisions (SOTA)
- **UI Architecture:** Expand upon the existing `AssistantPanel` component in GPUI. Introduce a 2-column layout cho Chat: Chat area (center) và Context Sources/Artifacts (right).
- **Data Persistence (Local-First):** Thay thế SurrealDB bằng SQLite local hoặc File-based JSON lưu trong `.omni/chat_sessions/` để đảm bảo tốc độ cực nhanh, tính riêng tư 100% và giảm dependency khi cài đặt (không cần daemon).
- **Context Window Management (RAG):** Thay vì nhồi toàn bộ nội dung web/PDF vào prompt gây tràn Token Limit, sẽ thiết lập một quy trình Local RAG. Dùng Vector DB thu nhỏ (`sqlite-vss` hoặc In-memory KD-Tree) để chia nhỏ và nhúng tài liệu (chunking & embedding). Agent sẽ truy vấn các chunk liên quan thay vì đọc toàn bộ.
- **Context Transfer Mechanism:** Nút "Transfer Context" sẽ kích hoạt agent chạy ngầm để tạo một bản "Executive Summary" kèm Reference IDs. Tóm tắt này được truyền qua dưới dạng system prompt ẩn vào không gian mới (Omni Design/IDE), giải quyết hoàn toàn vấn đề phình to ngữ cảnh (Context Bloat).

## Testing Decisions
- Mock external APIs (SearXNG, Web Fetcher) để unit test logic của Chat.
- Verify RAG pipeline bằng cách feed một file PDF 100 trang và test xem truy vấn có trích xuất đúng chunk hay không.
- Test "Transfer Context" bằng cách kiểm tra State của target agent có chứa đúng bản tóm tắt không.

## Out of Scope
- Direct code execution within Omni Chat (this belongs to Omni IDE).
- Real-time UI rendering (this belongs to Omni Design).
