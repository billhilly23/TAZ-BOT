// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@uniswap/v2-periphery/contracts/interfaces/IUniswapV2Router02.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract Sandwich is Ownable, ReentrancyGuard {
    IUniswapV2Router02 public uniswapRouter;

    // Event to log trades
    event TradeExecuted(address tokenIn, address tokenOut, uint256 amountIn, uint256 amountOut, string tradeType);

    constructor(address _uniswapRouter) {
        uniswapRouter = IUniswapV2Router02(_uniswapRouter);
    }

    // Approve tokens with a large allowance to avoid repeated approvals
    function approveTokenOnce(address tokenIn) external onlyOwner {
        require(
            IERC20(tokenIn).approve(address(uniswapRouter), type(uint256).max),
            "Token approval for Uniswap failed"
        );
    }

    // Execute a front-running trade with slippage management and profitability check
    function executeFrontRun(
        address tokenIn,
        address tokenOut,
        uint256 amountIn,
        uint256 slippageTolerancePercent,
        uint256 gasCostInTokens // Gas fee in terms of token cost
    ) external onlyOwner nonReentrant returns (uint256) {
        return executeTrade(tokenIn, tokenOut, amountIn, slippageTolerancePercent, gasCostInTokens, "front-run");
    }

    // Execute a back-running trade with slippage management and profitability check
    function executeBackRun(
        address tokenIn,
        address tokenOut,
        uint256 amountIn,
        uint256 slippageTolerancePercent,
        uint256 gasCostInTokens // Gas fee in terms of token cost
    ) external onlyOwner nonReentrant returns (uint256) {
        return executeTrade(tokenIn, tokenOut, amountIn, slippageTolerancePercent, gasCostInTokens, "back-run");
    }

    // Internal function to execute a trade with slippage management and profitability check
    function executeTrade(
        address tokenIn,
        address tokenOut,
        uint256 amountIn,
        uint256 slippageTolerancePercent,
        uint256 gasCostInTokens,
        string memory tradeType
    ) internal returns (uint256) {
        // Step 1: Simulate the trade to verify slippage tolerance
        uint256[] memory amountsOut = uniswapRouter.getAmountsOut(amountIn, getPathForTokenSwap(tokenIn, tokenOut));
        uint256 minAmountOut = amountsOut[1] * (100 - slippageTolerancePercent) / 100;

        // Step 2: Execute the trade on Uniswap
        uint256 amountOut = uniswapRouter.swapExactTokensForTokens(
            amountIn,
            minAmountOut,
            getPathForTokenSwap(tokenIn, tokenOut),
            address(this),
            block.timestamp + 120
        )[1];

        require(amountOut >= minAmountOut, "Trade failed");

        // Step 3: Calculate profit after gas cost and ensure it's positive
        uint256 profit = amountOut - amountIn;
        require(profit > gasCostInTokens, "Trade not profitable after gas cost");

        // Log the trade execution
        emit TradeExecuted(tokenIn, tokenOut, amountIn, amountOut, tradeType);

        return amountOut;
    }

    // Helper function to build the token swap path
    function getPathForTokenSwap(address tokenIn, address tokenOut)
        internal
        pure
        returns (address[] memory)
    {
        address;
        path[0] = tokenIn;
        path[1] = tokenOut;
        return path;
    }
}



