// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@uniswap/v2-periphery/contracts/interfaces/IUniswapV2Router02.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract HFT is Ownable, ReentrancyGuard {
    IUniswapV2Router02 public uniswapRouter;
    IUniswapV2Router02 public sushiswapRouter;

    // Event to log HFT execution
    event HFTExecuted(address tokenIn, address tokenOut, uint256 amountIn, uint256 amountOut, uint256 profit);

    constructor(address _uniswapRouter, address _sushiswapRouter) {
        uniswapRouter = IUniswapV2Router02(_uniswapRouter);
        sushiswapRouter = IUniswapV2Router02(_sushiswapRouter);
    }

    // Approve tokens with a large allowance to avoid repeated approvals
    function approveTokenOnce(address tokenIn, address tokenOut) external onlyOwner {
        require(
            IERC20(tokenIn).approve(address(uniswapRouter), type(uint256).max),
            "Token approval for Uniswap failed"
        );
        require(
            IERC20(tokenOut).approve(address(sushiswapRouter), type(uint256).max),
            "Token approval for Sushiswap failed"
        );
    }

    // Execute a high-frequency trade between Uniswap and Sushiswap with slippage management and profitability check
    function executeHFT(
        address tokenIn,
        address tokenOut,
        uint256 amountIn,
        uint256 slippageTolerancePercent,
        uint256 gasCostInTokens // Gas fee in terms of token cost
    ) external onlyOwner nonReentrant returns (bool) {
        // Step 1: Simulate the trade on Uniswap to verify slippage tolerance
        uint256[] memory amountsOutUniswap = uniswapRouter.getAmountsOut(amountIn, getPathForTokenSwap(tokenIn, tokenOut));
        uint256 minAmountOutUniswap = amountsOutUniswap[1] * (100 - slippageTolerancePercent) / 100;

        // Step 2: Swap on Uniswap if slippage is acceptable
        uint256 amountOutUniswap = uniswapRouter.swapExactTokensForTokens(
            amountIn,
            minAmountOutUniswap,
            getPathForTokenSwap(tokenIn, tokenOut),
            address(this),
            block.timestamp + 120
        )[1];

        require(amountOutUniswap >= minAmountOutUniswap, "Uniswap trade failed");

        // Step 3: Simulate the trade on Sushiswap to verify slippage tolerance
        uint256[] memory amountsOutSushiswap = sushiswapRouter.getAmountsOut(amountOutUniswap, getPathForTokenSwap(tokenOut, tokenIn));
        uint256 minAmountOutSushiswap = amountsOutSushiswap[1] * (100 - slippageTolerancePercent) / 100;

        // Step 4: Swap on Sushiswap if slippage is acceptable
        uint256 amountOutSushiswap = sushiswapRouter.swapExactTokensForTokens(
            amountOutUniswap,
            minAmountOutSushiswap,
            getPathForTokenSwap(tokenOut, tokenIn),
            address(this),
            block.timestamp + 120
        )[1];

        require(amountOutSushiswap >= minAmountOutSushiswap, "Sushiswap trade failed");

        // Step 5: Calculate profit after gas cost and ensure it's positive
        uint256 profit = amountOutSushiswap - amountIn;
        require(profit > gasCostInTokens, "HFT not profitable after gas cost");

        // Step 6: Transfer profit to the contract owner
        IERC20(tokenIn).transfer(owner(), profit - gasCostInTokens);

        // Log the HFT trade details
        emit HFTExecuted(tokenIn, tokenOut, amountIn, amountOutSushiswap, profit);

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



