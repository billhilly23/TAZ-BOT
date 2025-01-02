// SPDX-License-Identifier: MIT
pragma solidity ^0.5.11;

import "@uniswap/v2-periphery/contracts/interfaces/IUniswapV2Router02.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
contract Arbitrage is Ownable, ReentrancyGuard {
    IUniswapV2Router02 public uniswapRouter;
    IUniswapV2Router02 public sushiswapRouter;

    // Event to log arbitrage execution
    event ArbitrageExecuted(address tokenIn, address tokenOut, uint256 profit);

    constructor(address _uniswapRouter, address _sushiswapRouter) public {
        uniswapRouter = IUniswapV2Router02(_uniswapRouter);
        sushiswapRouter = IUniswapV2Router02(_sushiswapRouter);
    }

    // Function to approve tokens with a large allowance, minimizing repeated approvals
    function approveTokensOnce(address tokenIn, address tokenOut) external onlyOwner {
        require(
            IERC20(tokenIn).approve(address(uniswapRouter), type(uint256).max),
            "Token approval for Uniswap failed"
        );
        require(
            IERC20(tokenOut).approve(address(sushiswapRouter), type(uint256).max),
            "Token approval for Sushiswap failed"
        );
    }

    // Function to execute arbitrage with slippage tolerance
    function executeArbitrage(
        address tokenIn,
        address tokenOut,
        uint256 amountIn,
        uint256 slippageTolerancePercent,
        uint256 gasCostInTokens // Gas fee in terms of token cost
    ) external onlyOwner nonReentrant returns (bool) {
        // Step 1: Calculate minAmountOutUniswap with slippage tolerance
        uint256 uniswapPrice = getExpectedPriceFromUniswap(tokenIn, tokenOut, amountIn);
        uint256 minAmountOutUniswap = uniswapPrice * (100 - slippageTolerancePercent) / 100;

        // Step 2: Swap on Uniswap
        uint256 amountOutUniswap = uniswapRouter.swapExactTokensForTokens(
            amountIn,
            minAmountOutUniswap,
            getPathForTokenSwap(tokenIn, tokenOut),
            address(this),
            now + 120
        )[1];

        require(amountOutUniswap >= minAmountOutUniswap, "Uniswap trade failed");

        // Step 3: Calculate minAmountOutSushiswap with slippage tolerance
        uint256 sushiswapPrice = getExpectedPriceFromSushiswap(tokenOut, tokenIn, amountOutUniswap);
        uint256 minAmountOutSushiswap = sushiswapPrice * (100 - slippageTolerancePercent) / 100;

        // Step 4: Swap on Sushiswap
        uint256 amountOutSushiswap = sushiswapRouter.swapExactTokensForTokens(
            amountOutUniswap,
            minAmountOutSushiswap,
            getPathForTokenSwap(tokenOut, tokenIn),
            address(this),
            now + 120
        )[1];

        require(amountOutSushiswap >= minAmountOutSushiswap, "Sushiswap trade failed");

        // Step 5: Calculate profit after gas cost and ensure it's positive
        uint256 profit = amountOutSushiswap - amountIn;
        require(profit > gasCostInTokens, "Arbitrage not profitable after gas cost");

        // Step 6: Transfer profit to the contract owner
        IERC20(tokenIn).transfer(owner(), profit - gasCostInTokens);

        // Emit the arbitrage execution event
        emit ArbitrageExecuted(tokenIn, tokenOut, profit);

        return true;
    }
    // Helper function to calculate expected Uniswap price
    function getExpectedPriceFromUniswap(address tokenIn, address tokenOut, uint256 amountIn)
        internal
        view
        returns (uint256)
    {
        uint256[] memory amounts = uniswapRouter.getAmountsOut(amountIn, getPathForTokenSwap(tokenIn, tokenOut));
        return amounts[1]; // Return the expected output amount
    }

    // Helper function to calculate expected Sushiswap price
    function getExpectedPriceFromSushiswap(address tokenIn, address tokenOut, uint256 amountIn)
        internal
        view
        returns (uint256)
    {
        uint256[] memory amounts = sushiswapRouter.getAmountsOut(amountIn, getPathForTokenSwap(tokenIn, tokenOut));
        return amounts[1]; // Return the expected output amount
    }

    // Helper function to get the token swap path
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


