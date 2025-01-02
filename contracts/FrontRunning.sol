// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@uniswap/v2-periphery/contracts/interfaces/IUniswapV2Router02.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract FrontRunning is Ownable, ReentrancyGuard {
    IUniswapV2Router02 public uniswapRouter;

    // Event to log front-run execution
    event FrontRunExecuted(address tokenIn, address tokenOut, uint256 amountIn, uint256 amountOut, uint256 profit);

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
    ) external onlyOwner nonReentrant returns (bool) {
        // Step 1: Simulate the trade to verify slippage tolerance
        uint256[] memory amountsOut = uniswapRouter.getAmountsOut(amountIn, getPathForTokenSwap(tokenIn, tokenOut));
        uint256 minAmountOut = amountsOut[1] * (100 - slippageTolerancePercent) / 100;
        require(amountsOut[1] >= minAmountOut, "Price slippage too high");

        // Step 2: Execute the front-running trade on Uniswap
        uint256 amountOut = uniswapRouter.swapExactTokensForTokens(
            amountIn,
            minAmountOut,
            getPathForTokenSwap(tokenIn, tokenOut),
            address(this),
            block.timestamp + 120
        )[1];

        require(amountOut >= minAmountOut, "Front-run trade failed");

        // Step 3: Calculate profit after gas cost and ensure it's positive
        uint256 profit = amountOut - amountIn;
        require(profit > gasCostInTokens, "Front-run not profitable after gas cost");

        // Step 4: Transfer profit to the contract owner
        IERC20(tokenIn).transfer(owner(), profit - gasCostInTokens);

        // Log the front-run execution
        emit FrontRunExecuted(tokenIn, tokenOut, amountIn, amountOut, profit);

        return true;
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

