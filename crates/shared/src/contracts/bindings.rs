// 使用 alloy::sol! 宏生成绑定
use alloy::sol;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    TicketManager,
    "../../crates/shared/abis/TicketManager.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    EventManager,
    "../../crates/shared/abis/EventManager.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Marketplace,
    "../../crates/shared/abis/Marketplace.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    TokenSwap,
    "../../crates/shared/abis/TokenSwap.json"
);
