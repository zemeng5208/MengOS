//! feather-core：MengOS 的基础数据结构。
//!
//! 这里只放最小、最稳定的东西，供其他 MengOS 组件（例如 feather-terminal）复用。
//! 当前包含：任务标识、Bot 标识，以及任务状态机的转移规则。

use serde::{Deserialize, Serialize};
use std::fmt;

/// 任务标识。包一层字符串，避免和别的 ID 混用。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(String);

impl TaskId {
    /// 用任意能转成 `String` 的值创建任务标识。
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// 以字符串切片的形式借用内部值。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Bot 标识。同样包一层字符串。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BotId(String);

impl BotId {
    /// 用任意能转成 `String` 的值创建 Bot 标识。
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// 以字符串切片的形式借用内部值。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 任务状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskState {
    /// 排队中，还没开始。
    Queued,
    /// 正在执行。
    Running,
    /// 已成功结束。
    Succeeded,
    /// 已失败结束。
    Failed,
    /// 被取消。
    Cancelled,
}

/// 全部状态，方便遍历（例如测试与界面枚举）。
pub const ALL_STATES: [TaskState; 5] = [
    TaskState::Queued,
    TaskState::Running,
    TaskState::Succeeded,
    TaskState::Failed,
    TaskState::Cancelled,
];

impl TaskState {
    /// 判断状态机是否允许从当前状态转移到 `next`。
    ///
    /// 允许的转移只有：
    /// - 排队 → 执行、排队 → 取消
    /// - 执行 → 成功、执行 → 失败、执行 → 取消
    ///
    /// 终态（成功 / 失败 / 取消）不能再转到任何状态。
    pub fn can_transition_to(&self, next: &TaskState) -> bool {
        use TaskState::{Cancelled, Failed, Queued, Running, Succeeded};

        matches!(
            (self, next),
            (Queued, Running | Cancelled) | (Running, Succeeded | Failed | Cancelled)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queued_can_start_or_cancel() {
        assert!(TaskState::Queued.can_transition_to(&TaskState::Running));
        assert!(TaskState::Queued.can_transition_to(&TaskState::Cancelled));
    }

    #[test]
    fn queued_cannot_finish_directly() {
        assert!(!TaskState::Queued.can_transition_to(&TaskState::Succeeded));
        assert!(!TaskState::Queued.can_transition_to(&TaskState::Failed));
    }

    #[test]
    fn running_can_finish_or_cancel() {
        assert!(TaskState::Running.can_transition_to(&TaskState::Succeeded));
        assert!(TaskState::Running.can_transition_to(&TaskState::Failed));
        assert!(TaskState::Running.can_transition_to(&TaskState::Cancelled));
    }

    #[test]
    fn running_cannot_go_back_to_queued() {
        assert!(!TaskState::Running.can_transition_to(&TaskState::Queued));
    }

    #[test]
    fn terminal_states_are_terminal() {
        let terminal = [
            TaskState::Succeeded,
            TaskState::Failed,
            TaskState::Cancelled,
        ];
        for state in terminal {
            for next in ALL_STATES {
                assert!(
                    !state.can_transition_to(&next),
                    "{state:?} 是终态，不应能转移到 {next:?}"
                );
            }
        }
    }

    #[test]
    fn ids_display_as_inner_value() {
        let task = TaskId::new("task-42");
        assert_eq!(task.to_string(), "task-42");
        assert_eq!(task.as_str(), "task-42");

        let bot = BotId::new("bot-7");
        assert_eq!(bot.to_string(), "bot-7");
        assert_eq!(bot.as_str(), "bot-7");
    }

    #[test]
    fn ids_compare_by_value() {
        assert_eq!(TaskId::new("a"), TaskId::new("a"));
        assert_ne!(TaskId::new("a"), TaskId::new("b"));
        assert_ne!(BotId::new("a"), BotId::new("b"));
    }
}
