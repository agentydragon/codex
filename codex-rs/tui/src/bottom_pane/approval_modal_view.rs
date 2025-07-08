use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::WidgetRef;

use crate::app_event_sender::AppEventSender;
use crate::user_approval_widget::ApprovalRequest;
use crate::user_approval_widget::UserApprovalWidget;
use tracing::debug;

use super::BottomPane;
use super::BottomPaneView;

/// Modal overlay asking the user to approve/deny a sequence of requests.
#[derive(Debug)]
pub(crate) struct ApprovalModalView<'a> {
    current: UserApprovalWidget<'a>,
    queue: Vec<ApprovalRequest>,
    app_event_tx: AppEventSender,
    /// Color settings for TUI elements.
    colors: codex_core::config_types::Colors,
}

impl ApprovalModalView<'_> {
    pub fn new(
        request: ApprovalRequest,
        app_event_tx: AppEventSender,
        colors: codex_core::config_types::Colors,
    ) -> Self {
        debug!("UI: enqueue approval request: {:?}", request);
        Self {
            current: UserApprovalWidget::new(request, app_event_tx.clone(), colors.clone()),
            queue: Vec::new(),
            app_event_tx,
            colors,
        }
    }

    pub fn enqueue_request(&mut self, req: ApprovalRequest) {
        self.queue.push(req);
    }

    /// Advance to next request if the current one is finished.
    fn maybe_advance(&mut self) {
        if self.current.is_complete()
            && let Some(next) = self.queue.pop()
        {
            debug!("UI: advancing to next approval request: {:?}", next);
            self.current =
                UserApprovalWidget::new(next, self.app_event_tx.clone(), self.colors.clone());
        }
    }
}

impl<'a> BottomPaneView<'a> for ApprovalModalView<'a> {
    fn handle_key_event(&mut self, _pane: &mut BottomPane<'a>, key_event: KeyEvent) {
        self.current.handle_key_event(key_event);
        self.maybe_advance();
    }

    fn is_complete(&self) -> bool {
        self.current.is_complete() && self.queue.is_empty()
    }

    fn calculate_required_height(&self, area: &Rect) -> u16 {
        self.current.get_height(area)
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        (&self.current).render_ref(area, buf);
    }

    fn try_consume_approval_request(&mut self, req: ApprovalRequest) -> Option<ApprovalRequest> {
        self.enqueue_request(req);
        None
    }
}
