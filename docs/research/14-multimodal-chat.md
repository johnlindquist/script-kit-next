# Multimodal Chat UX Patterns: Images and Files

Last updated: 2026-02-01
Scope: Chat UIs with image and file inputs. Focus on image pasting, file attachments, drag-and-drop, and preview displays.

## Executive summary

Main patterns observed across ChatGPT, Claude, Gemini, Copilot, and Perplexity:

- A single attach control (plus or paperclip) near the composer is the primary entry point for files and images. [1][3][4][5][6][7]
- Drag-and-drop is a common secondary path for images and files (especially on desktop). [1][3][6][7]
- Clipboard image pasting is supported by at least some leading chats (ChatGPT, Claude). [1][3]
- Attachments show a visible state before or after sending (attachment item in the chat or a file icon in the input area). [5][8]
- Upload limits (file size, file count) are documented and should surface in UX as constraints and errors. [2][4]

## Pattern map (what users expect)

### 1) Entry point: attach control in the composer

- ChatGPT: image inputs can be added via an attach icon. [1]
- Claude: a plus button is used to add files or photos. [3]
- Gemini (web): files are added via an Add files control. [4]
- Copilot: a plus icon is used to add images or files. [5]
- Perplexity: a + Attach control is used to upload files and images. [6][7]

UX implication: users look for a single, consistent attach affordance next to the input field. Most services reuse it for both images and documents.

### 2) Image pasting (clipboard)

- ChatGPT: images can be pasted into the composer. [1]
- Claude: images can be pasted from the clipboard. [3]

UX implication: paste should work as a fast path, especially for screenshots. Provide instant feedback and a visible attachment state.

### 3) Drag-and-drop

- ChatGPT: images can be dragged and dropped into the composer. [1]
- Claude: files and photos can be dragged and dropped. [3]
- Perplexity: files and images can be dragged and dropped into the search bar. [6][7]

UX implication: drag-and-drop is expected on desktop, but it should be optional and not required for uploads.

### 4) Attachment preview and management

- Copilot: once a file is uploaded, it appears as an attachment in the chat session. [5]
- Perplexity: the file icon appears in the search bar and an X control is used to remove the file. [8]

UX implication: users expect a visible attachment state before sending or within the message thread, plus a clear remove action.

### 5) Limits and constraints

- ChatGPT documents explicit size limits (for files and images) and upload caps. [2]
- Gemini Apps state a maximum file count per prompt. [4]

UX implication: surface limits near the attach control, and provide clear inline errors when a file exceeds constraints.

## Product snapshots (evidence)

### ChatGPT

- Attach entry point: images can be added by selecting the attach icon. [1]
- Drag-and-drop images: supported. [1]
- Paste images: supported. [1]
- File limits: detailed size caps for files and images, plus upload caps. [2]

### Claude

- Attach entry point: plus button to add files or photos. [3]
- Drag-and-drop files and photos: supported. [3]
- Paste images from clipboard: supported. [3]

### Gemini (Gemini Apps)

- Attach entry point: Add files control for uploading files. [4]
- File count constraint: maximum number of files per prompt is specified in the help docs. [4]

### Microsoft Copilot

- Attach entry point: plus icon to add images or files. [5]
- Preview display: uploaded files appear as attachments in the chat session. [5]

### Perplexity

- Attach entry point: + Attach control for files; attach icon for images. [6][7]
- Drag-and-drop: files and images can be dragged into the search bar. [6][7]
- Preview and removal: file icon appears in the search bar, with an X to remove. [8]

## Design takeaways for multimodal chat

- Primary attach affordance should be obvious and near the composer, not hidden in menus.
- Provide at least two paths: attach button + drag-and-drop; add clipboard paste for images when possible.
- Use an explicit attachment state (chip, file icon, or inline attachment row) and provide a one-click remove action.
- Documented limits should be mirrored in UI copy and error handling (size caps, file count caps).

## Gaps and follow-up questions

- Are there best-in-class patterns for multi-file preview galleries (thumbnails, file metadata, or reorder) that are not covered by the official help docs?
- How do these products handle attachment previews on mobile vs desktop (not covered in these sources)?

## Sources

[1] OpenAI Help Center, "ChatGPT Image Inputs FAQ" (attach icon, drag and drop, paste). https://help.openai.com/en/articles/8400551-chatgpt-image-inputs-faq
[2] OpenAI Help Center, "File Uploads FAQ" (size limits, upload caps). https://help.openai.com/en/articles/8555545
[3] Anthropic Help Center, "Uploading Files to Claude" (plus button, drag and drop, paste). https://support.anthropic.com/en/articles/8624994-how-to-upload-files-and-images-to-claude
[4] Google Gemini Apps Help, "Upload and analyse files in Gemini Apps" (Add files control, file count). https://support.google.com/gemini/answer/14903178
[5] Microsoft Copilot Support, "Add images or files in Copilot" (plus icon, attachment in chat). https://support.microsoft.com/en-us/topic/add-images-or-files-in-copilot-0ba5a8e8-2b7a-4cf2-9296-27f65a7f08bf
[6] Perplexity Help Center, "File Uploads FAQ" (+ Attach control, drag and drop). https://www.perplexity.ai/help-center/faq/file-uploads-faq
[7] Perplexity Help Center, "Uploading images to Perplexity" (attach icon, drag and drop). https://www.perplexity.ai/help-center/faq/uploading-images-to-perplexity
[8] Perplexity Help Center, "What data do you retain about me?" (file icon in search bar, X to remove). https://www.perplexity.ai/help-center/faq/what-data-do-you-retain-about-me
