# Omni Design (The Prototyper) PRD

## Problem Statement
Việc chuyển từ ý tưởng (Spec/PRD) sang code UI thường tốn thời gian vì developer phải chuyển đổi liên tục giữa code và trình duyệt để xem kết quả. Cần một công cụ cho phép prompt ra giao diện trực quan và xem ngay kết quả như Claude Artifacts.

## Solution
Tạo một không gian (Omni Design) tích hợp trình duyệt xem trước ngay trong Omni IDE. Giao diện chia làm hai phần: Chat (trái) và Canvas/Preview (phải). Agent sẽ tạo ra code UI (React/Tailwind/HTML) và nó sẽ được render ngay lập tức, cho phép người dùng lặp lại thiết kế nhanh chóng trước khi đẩy vào codebase thật.

## User Stories
1. As a developer, I want to chat with a UI Designer agent on the left panel, so that I can request UI components.
2. As a designer, I want to see the live rendering of the requested UI component on the right panel, so that I can instantly evaluate its look and feel.
3. As a developer, I want to switch between the 'Preview' and 'Code' tabs on the right panel, so that I can inspect the generated React/Tailwind code.
4. As a developer, I want to click an "Export to IDE Workspace" button, so that the generated and approved UI code is automatically placed into my actual project directory.

## Technical & Architecture Decisions (SOTA)
- **WebView Integration (Wry vs GPUI):** Giải quyết triệt để xung đột z-index giữa GPUI (vẽ bằng GPU) và Native WebView bằng cách: Preview Canvas sẽ là một **Native Child Window** độc lập neo (dock) sát vào cửa sổ chính thay vì nhúng trực tiếp, hoặc sử dụng Platform View Layering nếu hệ điều hành hỗ trợ tốt. Điều này đảm bảo các tooltip/menu GPUI không bị WebView nuốt hiển thị.
- **Rendering Pipeline (React + Tailwind):** Tránh dùng IPC để truyền chuỗi code React lớn thẳng vào WebView (dễ giật lag và lỗi escape string). Thay vào đó, khởi tạo một **Local HTTP Server (bằng Axum/Actix)** nội bộ tại một port động (vd: 34567). WebView đóng vai trò như một trình duyệt thông thường truy cập `http://localhost:34567`.
- **Hot Module Replacement (HMR) & Styling:** Sử dụng `esbuild` dạng in-memory và nhúng `Twind` (Tailwind-in-JS) để biên dịch React+Tailwind ngay trong quá trình runtime. Local Server sẽ đẩy tín hiệu HMR qua WebSockets, giúp Canvas refresh UI mượt mà như trải nghiệm dùng Vite (không cần reload trang).
- **IPC Bridge:** Chỉ dùng để truyền các lệnh điều khiển (như: click "Export", "Refresh") thay vì truyền data nặng.

## Testing Decisions
- Unit test Local HTTP Server để đảm bảo nó phục vụ chính xác các file HTML/JS được sinh ra từ bộ nhớ ảo.
- End-to-end (E2E) test: Gửi một mock React component, kiểm tra Local Server có build thành công và trả về mã 200 không.
- Test kiến trúc Child Window docking khi resize cửa sổ IDE chính.

## Out of Scope
- Full application routing and state management within the prototype.
- Backend database connections from within the WebView prototype.
