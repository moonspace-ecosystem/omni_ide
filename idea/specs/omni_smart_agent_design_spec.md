# Omni Smart Agent Design (The Orchestrator) PRD

## Problem Statement
Khi dự án phức tạp lên, một agent đơn lẻ không thể hoàn thành toàn bộ khối lượng công việc. Developer cần khả năng tổ chức các Agent thành một "Binh đoàn" (Swarm) và thiết lập luồng dữ liệu (DAG) giữa chúng một cách trực quan để xử lý các task phức tạp một cách tự động.

## Solution
Tạo một không gian thiết kế dạng Node-based (tương tự n8n hoặc ComfyUI) bên trong Omni IDE. Người dùng có thể kéo thả các Agent (dựa trên các Skill có sẵn), nối các đầu ra/đầu vào với nhau tạo thành một biểu đồ DAG, và thiết lập cấu hình Prompt/LLM cho từng node. Cuối cùng xuất ra file để chạy DAG.

## User Stories
1. As a developer, I want to see a canvas editor with draggable nodes, so that I can visually arrange my agent workflows.
2. As a developer, I want to drag connections between nodes, so that I can pass the output of one agent as the input to another.
3. As a developer, I want a sidebar showing all available Skills, so that I can drag a specific skill onto the canvas to create a new agent node.
4. As a developer, I want to click on a node to open a Property Inspector, so that I can customize the prompt, LLM model, and parameters for that specific agent.
5. As a developer, I want to compile the visual graph into a `plan.json` file, so that I can execute the swarm using `omni run --dag`.

## Technical & Architecture Decisions (SOTA)
- **Node-based Canvas Editor (GPUI Custom Element):** Xây dựng một custom component dùng `gpui::Render`. Việc vẽ đường nối (Bezier curves) sẽ được thực hiện bằng `gpui::Path` builder. Các thao tác Hit-testing (nhận diện click, kéo dây, chọn node) sẽ được xử lý bằng thuật toán bounding-box và khoảng cách hình học nội bộ.
- **Separation of State (Visual vs Logical):** Quản lý trạng thái bằng 2 định dạng schema tách biệt hoàn toàn để tối ưu:
  - `workspace_state.json`: Chứa các siêu dữ liệu đồ họa (Toạ độ X,Y của các Node, mức độ Pan/Zoom của Canvas).
  - `plan.json`: Chứa Swarm DAG Schema thuần tuý (chỉ tập trung vào Logic liên kết) dùng làm input cho Engine thực thi.
- **Cycle Detection & Validation (Chống vòng lặp):** Canvas tích hợp thuật toán DFS (Depth-First Search) chạy ngầm (debounced). Ngay khi người dùng kéo một dây nối gây ra vòng lặp vô tận (VD: Node A -> Node B -> Node A), dây sẽ chuyển màu đỏ và từ chối kết nối, đảm bảo DAG luôn hợp lệ (Acyclic).
- **Skill Type-Safety:** Engine sẽ đọc thư mục `SKILL.md` nội bộ, parse YAML frontmatter để tự động sinh ra các Input/Output port (cổng kết nối) đúng chuẩn trên từng Node. Hệ thống sẽ báo lỗi khi nối sai kiểu dữ liệu.

## Testing Decisions
- Unit test thuật toán Cycle Detection (DFS) với các cấu trúc đồ thị phức tạp để đảm bảo không bị crash UI.
- Unit test hàm ánh xạ (Compiler) từ nội bộ Visual State sang `plan.json`.
- Tích hợp test tự động để đảm bảo các file `SKILL.md` render ra đúng số lượng Input/Output port trên Canvas.

## Out of Scope
- Visual debugging of executing nodes in real-time (Phase 1 will only support generation of the plan; real-time execution tracking will be Phase 2).
