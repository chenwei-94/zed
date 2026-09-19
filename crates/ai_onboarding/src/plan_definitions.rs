use gpui::{IntoElement, ParentElement};
use ui::{List, ListBulletItem, prelude::*};

/// Centralized definitions for Zed AI plans
pub struct PlanDefinitions;

impl PlanDefinitions {
    pub fn free_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new("2,000 次已接受的编辑预测"))
            .child(ListBulletItem::new(
                "使用你的 AI API 密钥的无限提示词",
            ))
            .child(ListBulletItem::new("Unlimited use of external agents"))
    }

    pub fn sign_in_upsell(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new("无限制的编辑预测"))
            .child(ListBulletItem::new("$5 of GPT Luna"))
            .child(ListBulletItem::new("无需信用卡"))
    }

    pub fn pro_trial(&self, period: bool) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new("$5 of GPT Luna"))
            .child(ListBulletItem::new("无限制的编辑预测"))
            .when(period, |this| {
                this.child(ListBulletItem::new(
                    "自试用开始起 14 天，无需信用卡",
                ))
            })
    }

    pub fn pro_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new("$5 of tokens in Zed agent"))
            .child(ListBulletItem::new("超出 $5 的按用量计费"))
            .child(ListBulletItem::new("无限制的编辑预测"))
    }

    pub fn business_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new("无限制的编辑预测"))
            .child(ListBulletItem::new("按用量计费"))
    }

    pub fn vip_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new("无限制的编辑预测"))
            .child(ListBulletItem::new("Zed 智能体中的 token"))
    }

    pub fn student_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new("无限制的编辑预测"))
            .child(ListBulletItem::new("$10 of tokens in Zed agent"))
            .child(ListBulletItem::new(
                "可选的额度包，用于额外用量",
            ))
    }
}
