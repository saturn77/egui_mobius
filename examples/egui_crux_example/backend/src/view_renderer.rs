use crux_core::capability::{CapabilityContext, Operation};
use crux_core::{Command, Request};
use crux_core::macros::Capability;
use crate::View;

#[derive(Capability)]
pub struct ViewRenderer<Ev> {
    context: CapabilityContext<ViewRendererOperation, Ev>,
}

impl<Ev> ViewRenderer<Ev> {
    pub fn new(context: CapabilityContext<ViewRendererOperation, Ev>) -> Self {
        Self {
            context,
        }
    }
}
impl<Ev: 'static> ViewRenderer<Ev> {
    pub fn view(&self, view: View) {
        self.context.spawn({
            let context = self.context.clone();
            async move {
                run_view(&context, view).await;
            }
        });
    }
}

async fn run_view<Ev: 'static>(context: &CapabilityContext<ViewRendererOperation, Ev>, view: View) {
    context
        .notify_shell(ViewRendererOperation::View {
            view,
        })
        .await
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum ViewRendererOperation {
    View { view: View },
}

impl Operation for ViewRendererOperation {
    type Output = ();
}

pub fn view<Effect, Event>(view: View) -> Command<Effect, Event>
where
    Effect: From<Request<ViewRendererOperation>> + Send + 'static,
    Event: Send + 'static,
{
    Command::notify_shell(crate::view_renderer::ViewRendererOperation::View { view })
}
