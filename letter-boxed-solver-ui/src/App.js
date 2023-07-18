import React from "react";

import * as wasm from "letter-boxed-solver-wasm";

import "./App.css";

await wasm;

function App() {
  const [solving, setSolving] = React.useState(false);
  const [solution, setSolution] = React.useState("");
  const [priorWords, setPriorWords] = React.useState("");
  const [side1, setSide1] = React.useState("");
  const [side2, setSide2] = React.useState("");
  const [side3, setSide3] = React.useState("");
  const [side4, setSide4] = React.useState("");
  const [depth, setDepth] = React.useState(2);

  return (
    <div className="App">
      <h1>Letter Boxed Solver</h1>
      <p>
        <a
          href="https://www.nytimes.com/puzzles/letter-boxed"
          target="_blank"
          rel="noreferrer"
        >
          Letter Boxed on NYT Games
        </a>
      </p>
      <p>
        Type the four sides of the Letter Boxed Puzzle here, and the maximum
        number of words
      </p>
      <p>
        <input
          type="text"
          placeholder="top"
          value={side1}
          onChange={(evt) => setSide1(evt.target.value.toUpperCase())}
        />
        <input
          type="text"
          placeholder="right"
          value={side2}
          onChange={(evt) => setSide2(evt.target.value.toUpperCase())}
        />
        <input
          type="text"
          placeholder="bottom"
          value={side3}
          onChange={(evt) => setSide3(evt.target.value.toUpperCase())}
        />
        <input
          type="text"
          placeholder="left"
          value={side4}
          onChange={(evt) => setSide4(evt.target.value.toUpperCase())}
        />
        <input
          type="number"
          value={depth}
          onChange={(evt) => setDepth(parseInt(evt.target.value, 10))}
        />
      </p>
      <p>
        If you have a prefix of words you want to start with, enter them
        here:&nbsp;
        <input
          type="text"
          placeholder=""
          value={priorWords}
          onChange={(evt) => setPriorWords(evt.target.value.toUpperCase())}
        />
      </p>
      <p>
        <button
          onClick={() => {
            setSolving(true);
            setTimeout(() => {
              setSolution(
                wasm.solve(side1, side2, side3, side4, priorWords, depth)
              );
              setSolving(false);
            }, 0);
          }}
          disabled={
            side1.length === 0 ||
            side2.length === 0 ||
            side3.length === 0 ||
            side4.length === 0 ||
            solving
          }
        >
          Solve
        </button>
      </p>
      <pre>{solution}</pre>
    </div>
  );
}

export default App;
