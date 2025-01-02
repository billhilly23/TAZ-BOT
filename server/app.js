const express = require('express');
const cors = require('cors');
const morgan = require('morgan');
const arbitrageRoute = require('./routes/arbitrage');
const flashloanRoute = require('./routes/flashloan');
const frontrunningRoute = require('./routes/frontrunning');
const sandwichRoute = require('./routes/sandwich');
const liquidationRoute = require('./routes/liquidation');
const hftRoute = require('./routes/hft');

const app = express();
app.use(express.json());
app.use(cors());
app.use(morgan('dev'));

// Routes for each module
app.use('/run-arbitrage', arbitrageRoute);
app.use('/run-flashloan', flashloanRoute);
app.use('/run-frontrunning', frontrunningRoute);
app.use('/run-sandwich', sandwichRoute);
app.use('/run-liquidation', liquidationRoute);
app.use('/run-hft', hftRoute);

// Error handling middleware
app.use((err, req, res, next) => {
    console.error(err.stack);
    res.status(500).json({ message: 'Something went wrong!' });
});

// Start the server
const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
    console.log(`Server running on port ${PORT}`);
});

