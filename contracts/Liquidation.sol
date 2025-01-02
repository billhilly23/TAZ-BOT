// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@aave/protocol-v2/contracts/interfaces/ILendingPool.sol";
import "@aave/protocol-v2/contracts/interfaces/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "./interfaces/IComptroller.sol";  // Import Compound Comptroller interface
import "./interfaces/ICToken.sol";       // Import Compound cToken interface
import "@chainlink/contracts/src/v0.8/interfaces/AggregatorV3Interface.sol"; // Chainlink for price feeds

contract Liquidation is Ownable {
    ILendingPool public aavePool;
    IComptroller public comptroller;
    address public cTokenCollateral;

    AggregatorV3Interface internal priceFeed; // Chainlink price feed for collateral asset prices

    constructor(address _aavePool, address _comptroller, address _cTokenCollateral, address _priceFeed) {
        aavePool = ILendingPool(_aavePool);
        comptroller = IComptroller(_comptroller);
        cTokenCollateral = _cTokenCollateral;
        priceFeed = AggregatorV3Interface(_priceFeed); // Initialize Chainlink price feed
    }

    // Approve tokens with a large allowance to avoid repeated approvals
    function approveTokenOnce(address debtAsset) external onlyOwner {
        require(
            IERC20(debtAsset).approve(address(aavePool), type(uint256).max),
            "Token approval for Aave Pool failed"
        );
    }

    // Execute a liquidation on Aave or Compound with profitability and gas consideration
    function executeLiquidation(
        address borrower,
        uint256 debtToCover,
        address collateralAsset,
        address debtAsset,
        bool receiveAToken,
        uint256 gasCostInTokens // Gas fee in terms of token cost
    ) external onlyOwner returns (bool) {
        IERC20(debtAsset).approve(address(aavePool), debtToCover);

        uint256 potentialProfit = estimateProfit(collateralAsset, debtToCover);
        require(potentialProfit > gasCostInTokens, "Liquidation not profitable after gas cost");

        aavePool.liquidationCall(collateralAsset, debtAsset, borrower, debtToCover, receiveAToken);

        return true;
    }

    // Compound-specific liquidation function with profitability and gas cost consideration
    function executeCompoundLiquidation(
        address borrower,
        address cTokenBorrowed,
        address cTokenCollateral,
        uint256 repayAmount,
        uint256 gasCostInTokens
    ) external onlyOwner {
        uint256 potentialProfit = estimateProfit(cTokenCollateral, repayAmount);
        require(potentialProfit > gasCostInTokens, "Compound liquidation not profitable after gas cost");

        IERC20(cTokenBorrowed).approve(cTokenBorrowed, repayAmount);

        require(ICToken(cTokenBorrowed).liquidateBorrow(borrower, repayAmount, cTokenCollateral) == 0, "Compound liquidation failed");

        uint256 seizeAmount = estimateSeizeAmount(cTokenCollateral, repayAmount);
        require(ICToken(cTokenCollateral).seize(address(this), borrower, seizeAmount) == 0, "Seizing collateral failed");
    }

    // Withdraw any leftover collateral tokens from the contract
    function withdrawProfits(address token) external onlyOwner {
        uint256 balance = IERC20(token).balanceOf(address(this));
        require(balance > 0, "No profits to withdraw");
        IERC20(token).transfer(owner(), balance);
    }

    // Estimate profit from liquidation by getting the price of the collateral
    function estimateProfit(address collateralAsset, uint256 debtToCover) internal view returns (uint256) {
        (, int256 price, , , ) = priceFeed.latestRoundData();  // Get latest price from Chainlink price feed
        uint256 collateralValue = uint256(price) * debtToCover / 1e8;  // Assuming 8 decimal places for price feed
        return collateralValue * 110 / 100;  // 10% profit estimate
    }

    // Calculate how much collateral to seize based on Compound's liquidation rules
    function estimateSeizeAmount(address cTokenCollateral, uint256 repayAmount) internal view returns (uint256) {
        uint256 liquidationIncentiveMantissa = comptroller.liquidationIncentiveMantissa();
        uint256 seizeTokens = repayAmount * liquidationIncentiveMantissa / 1e18;  // Compound uses 18 decimals
        return seizeTokens;
    }
}


