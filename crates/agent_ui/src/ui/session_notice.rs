use agent_client_protocol::schema::v1 as acp;
use component::{Component, ComponentScope, example_group_with_title, single_example};
use gpui::{AnyElement, App, ClickEvent, ElementId, Stateful, Window, px};
use ui::{Callout, Color, IconButton, IconName, IconSize, Severity, Tooltip, prelude::*};

#[derive(IntoElement)]
pub struct SessionNotice {
    id: ElementId,
    callout: Callout,
}

impl SessionNotice {
    pub fn new(
        id: impl Into<ElementId>,
        notice: &acp::Notice,
        on_dismiss: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        let id = id.into();
        let dismiss_id = id.clone();
        let (severity, icon) = match &notice.severity {
            acp::NoticeSeverity::Info => (Severity::Info, Some(IconName::Info)),
            acp::NoticeSeverity::Warning => (Severity::Warning, Some(IconName::Warning)),
            acp::NoticeSeverity::Error => (Severity::Error, Some(IconName::XCircle)),
            _ => (Severity::Info, None),
        };

        Self {
            id,
            callout: Callout::new()
                .severity(severity)
                .title(notice.title.clone())
                .when_some(icon, Callout::icon)
                .when_some(notice.description.clone(), Callout::description)
                .scrollable_description(false)
                .dismiss_action(
                    div()
                        .debug_selector(move || format!("dismiss-{dismiss_id}"))
                        .child(
                            IconButton::new("dismiss", IconName::Close)
                                .icon_size(IconSize::Small)
                                .icon_color(Color::Muted)
                                .aria_label(format!("关闭通知：{}", notice.title))
                                .tab_index(0_isize)
                                .tooltip(Tooltip::text("关闭通知"))
                                .on_click(on_dismiss),
                        ),
                ),
        }
    }
}

impl RenderOnce for SessionNotice {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div().id(self.id).w_full().flex_none().child(self.callout)
    }
}

pub fn session_notice_list(id: impl Into<ElementId>) -> Stateful<Div> {
    v_flex()
        .id(id)
        .w_full()
        .flex_shrink_0()
        .max_h(rems_from_px(192_f32))
        .overflow_y_scroll()
}

fn preview_notice(id: impl Into<ElementId>, notice: acp::Notice) -> AnyElement {
    SessionNotice::new(id, &notice, |_, _, _| {}).into_any_element()
}

#[derive(RegisterComponent)]
pub struct SessionNoticePreview;

impl Component for SessionNoticePreview {
    fn scope() -> ComponentScope {
        ComponentScope::Agent
    }

    fn name() -> &'static str {
        "Session Notice"
    }

    fn description() -> &'static str {
        "以智能体输入框上方的实际渲染效果展示 ACP 会话通知；这些预览中的关闭按钮仅供示意。"
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> AnyElement {
        let detailed = acp::Notice::new(acp::NoticeSeverity::Warning, "MCP 服务器不可用")
            .description("将在缺少它的情况下继续；其他集成仍然可用。");
        let long = acp::Notice::new(
            acp::NoticeSeverity::Warning,
            "可选的文档集成在此远程环境中不可用",
        )
        .description(
            "工作将继续使用你项目中的文件。\n智能体多次尝试后仍无法连接文档服务。\n**这是纯文本**，不是 Markdown。",
        );

        v_flex()
            .gap_6()
            .children([
                example_group_with_title(
                    "严重级别",
                    vec![
                        single_example(
                            "信息",
                            preview_notice(
                                "preview-info-notice",
                                acp::Notice::new(acp::NoticeSeverity::Info, "索引已恢复"),
                            ),
                        )
                        .width(px(640.)),
                        single_example(
                            "带详情的警告",
                            preview_notice("preview-warning-notice", detailed),
                        )
                        .width(px(640.)),
                        single_example(
                            "错误",
                            preview_notice(
                                "preview-error-notice",
                                acp::Notice::new(acp::NoticeSeverity::Error, "可选集成失败")
                                    .description("缺少此集成时工作仍将继续。"),
                            ),
                        )
                        .width(px(640.)),
                        single_example(
                            "未来的严重级别",
                            preview_notice(
                                "preview-custom-notice",
                                acp::Notice::new(
                                    acp::NoticeSeverity::Other("maintenance".into()),
                                    "计划维护",
                                ),
                            ),
                        )
                        .width(px(640.)),
                        single_example(
                            "自定义严重级别",
                            preview_notice(
                                "preview-extension-notice",
                                acp::Notice::new(
                                    acp::NoticeSeverity::Other("_agent_advisory".into()),
                                    "正在使用工作区配置",
                                )
                                .description("智能体专属与未来的严重级别使用同一套通用呈现。"),
                            ),
                        )
                        .width(px(640.)),
                    ],
                )
                .vertical()
                .into_any_element(),
                example_group_with_title(
                    "布局",
                    vec![
                        single_example("窄", preview_notice("preview-narrow-notice", long))
                            .width(px(320.)),
                        single_example(
                            "通知堆栈",
                            session_notice_list("preview-notice-stack")
                                .children([
                                    preview_notice(
                                        "preview-stack-info",
                                        acp::Notice::new(
                                            acp::NoticeSeverity::Info,
                                            "已连接到远程环境",
                                        ),
                                    ),
                                    preview_notice(
                                        "preview-stack-warning",
                                        acp::Notice::new(
                                            acp::NoticeSeverity::Warning,
                                            "网络性能下降",
                                        )
                                        .description("响应可能比平时更慢。"),
                                    ),
                                ])
                                .into_any_element(),
                        )
                        .width(px(640.)),
                        single_example(
                            "溢出通知堆栈",
                            session_notice_list("preview-overflow-notice-stack")
                                .children((0..6_usize).map(|index| {
                                    preview_notice(
                                        ("preview-repeated-notice", index),
                                        acp::Notice::new(
                                            acp::NoticeSeverity::Warning,
                                            "MCP 服务器不可用",
                                        )
                                        .description(
                                            "将在缺少它的情况下继续；重复通知会作为独立事件显示。",
                                        ),
                                    )
                                }))
                                .into_any_element(),
                        )
                        .width(px(320.)),
                    ],
                )
                .vertical()
                .into_any_element(),
            ])
            .into_any_element()
    }
}
