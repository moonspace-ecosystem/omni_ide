# Implementation Plan: Omni Trae-like Ecosystem & Smart Agent Orchestrator

## Goal Description
Biến đổi và mở rộng project `OmniUltraAgent_Kit` để tích hợp trực tiếp vào **Omni IDE**. Hệ thống sẽ được nâng cấp thành một hệ sinh thái AI Native Workspace với giao diện thống nhất gồm 4 trụ cột: **Omni Chat**, **Omni Design**, **Omni IDE**, và **Omni Smart Agent Design**. 

Quy trình hoạt động của hệ sinh thái sẽ tuân thủ nghiêm ngặt vòng đời phát triển phần mềm: 
`Research & Brainstorm -> Define (Spec) -> Plan + Task -> Build -> Verify -> Review -> Ship & Deploy.`

Đồng thời, hệ thống sẽ sử dụng kỹ năng `omni-server` (gọi thông qua slash command `/omni-server`) để Agent có thể tìm kiếm, sử dụng 1900+ skills chuyên gia và điều phối sub-agents thông qua `omni run`.

## User Review Required
> [!IMPORTANT]  
> 1. Tính năng **Omni Smart Agent Design (n8n style)** đòi hỏi xây dựng giao diện Node-based Editor. Nó sẽ biên dịch các node thành file JSON DAG (`Swarm DAG Schema`) mà lệnh `omni run --dag` đang hỗ trợ.
> 2. Giao diện tích hợp WebView cho Omni Design đòi hỏi sự giao tiếp phức tạp giữa Rust (GPUI) và macOS WebKit.

## Open Questions
1. Bạn muốn chạy `OmniUltraAgent_Kit` như một thư viện Rust tĩnh (`omni-kit` crate) gọi trực tiếp trong codebase của Omni IDE, hay gọi qua CLI sidecar? (Hiện tại chúng ta đã tích hợp thử dạng path dependency crate ở Phase 4 trước đó).
2. Với Omni Design, vì Omni IDE được viết bằng GPUI (Rust native) không có DOM HTML, chúng ta sẽ cần nhúng một WebView (như `wry` hoặc CEF) để render HTML/CSS/JS/React live preview. Hướng tiếp cận này có phù hợp với kiến trúc bạn mong muốn không?

---

## Vòng Đời Phát Triển (Lifecycle & The 4 Pillars)

### 1. Research & Brainstorm (Omni Chat)
- Chi tiết tham khảo: [Omni Chat Specification](file:///Users/mike/Documents/omni_ide/idea/specs/omni_chat_spec.md)
- Nơi diễn ra quá trình thu thập thông tin, nghiên cứu sâu (Deep Research).
- Sử dụng Slash Command `/omni-server` để gọi kỹ năng `omni-server` (hoặc lệnh `omni search-skills`) nhằm tìm kiếm các skill nghiên cứu phù hợp.
- **Công cụ:** Omni Chat Panel (hoạt động độc lập giống Claude Cowork).
- **Đầu ra:** Các báo cáo nghiên cứu, dữ liệu thị trường, ý tưởng thô.

### 2. Define (Spec) (Omni Chat / Omni Design)
- Chi tiết tham khảo: [Omni Design Specification](file:///Users/mike/Documents/omni_ide/idea/specs/omni_design_spec.md)
- Chuyển đổi ý tưởng thô thành Specification chi tiết (PRD).
- Sử dụng Omni Design để phác thảo giao diện trực quan (Wireframes, Live Preview HTML/React).
- **Knowledge Transfer:** Bấm nút chuyển toàn bộ context spec và design sang IDE.
- **Đầu ra:** File PRD (`.md`), Design Artifacts.

### 3. Plan + Task (Omni IDE & Omni Smart Agent Design)
- Chi tiết tham khảo: [Omni Smart Agent Design Specification](file:///Users/mike/Documents/omni_ide/idea/specs/omni_smart_agent_design_spec.md)
- AI phân rã Specification thành Implementation Plan và Task list (`task.md`).
- Người dùng có thể dùng **Omni Smart Agent Design** (giao diện kéo thả Node-based n8n-style) để thiết kế luồng (DAG) chia việc cho nhiều Agent khác nhau, tận dụng hệ thống `omni run --dag plan.json`.
- **Đầu ra:** Kế hoạch thực thi rõ ràng, sơ đồ DAG.

### 4. Build (Omni IDE)
- Giai đoạn lập trình cốt lõi (The Builder).
- Sử dụng hệ thống `AgenticRunner` (qua `omni run` hoặc native crate) để tự động sửa nhiều file cùng lúc, viết code, sinh components.
- Agent đóng vai trò Pair Programmer, thao tác trực tiếp trên source files thực sự.

### 5. Verify (Omni IDE)
- Xác minh code bằng cách chạy compiler (`cargo check`, `npm run build`), Unit Tests và Integration Tests.
- Lợi dụng các công cụ Code Intelligence (`omni impact`, `omni graph`) từ `omni-server` để kiểm tra ảnh hưởng của các thay đổi mới.
- **Đầu ra:** Kết quả test xanh, log build thành công.

### 6. Review (Omni IDE)
- Phân tích độ an toàn và kiến trúc của code vừa build. 
- Gọi lệnh `omni security scan` hoặc tạo luồng review code qua Agent. Trình diễn qua MultiDiffView trong Omni IDE (thừa kế từ GitButler panel).

### 7. Ship & Deploy (Omni IDE)
- Auto commit, sinh release notes, tự động push và trigger CI/CD pipeline.
- Hoàn thành quy trình bằng việc triển khai ứng dụng.

---

## Kế hoạch Triển khai (Proposed Changes)

Do yêu cầu thay đổi lớn về giao diện GPUI của Omni IDE, tiến trình được chia thành:

### Phase 1: Core Integration (Omni IDE x Omni Kit) - [ĐÃ HOÀN THÀNH]
- **[MODIFY]** `Cargo.toml`: Tích hợp thư viện `omni-kit` vào workspace.
- **[NEW]** Setup SurrealDB connection pool trong `AppState` toàn cục của GPUI.

### Phase 2: Hệ sinh thái Omni Chat & WebView Omni Design
- **[NEW]** Cấu trúc lại `AssistantPanel` thành không gian `OmniChat`.
- **[NEW]** Thêm tính năng WebView cho Omni IDE để phục vụ render `OmniDesign` (live preview).
- **[NEW]** Tích hợp tính năng Knowledge Transfer (luân chuyển context qua Memory Token).

### Phase 3: Công cụ Omni Smart Agent Design
- **[NEW]** Xây dựng Node Editor (GPUI drag & drop) đọc danh mục tools từ `omni tools list`.
- **[NEW]** Trình biên dịch: Graph (GPUI Nodes) -> `Swarm DAG Schema JSON`.

### Phase 4: IDE Execution Builder & Tooling
- **[MODIFY]** Cập nhật hệ thống File Diff, áp dụng AgenticRunner cho luồng Build & Verify.
- **[NEW]** Tích hợp slash command `/omni-server` trực tiếp vào thanh chat của IDE.

---

## Verification Plan

### Automated Tests
- Kiểm tra module `compile_canvas_to_dag` sinh JSON schema chính xác.
- Đảm bảo WebView bridge render HTML từ Rust sang hệ điều hành ổn định.

### Manual Verification
- Chạy end-to-end vòng đời phát triển: Bắt đầu từ **Research** trong Omni Chat, thiết kế spec trên **Omni Design**, phân rã **Plan**, tự động **Build** trong Omni IDE, **Verify** và cuối cùng **Deploy**. Mọi luồng thông tin phải được luân chuyển xuyên suốt.
