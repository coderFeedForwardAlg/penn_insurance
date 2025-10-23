import './App.css';
import { useState } from 'react';

function App() {
  const [question, setQuestion] = useState('');
  const [urls, setUrls] = useState([]);
  const [answer, setAnswer] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (!question.trim()) return;

    setIsLoading(true);
    setAnswer('');

    try {
      const response = await fetch(`${process.env.REACT_APP_APIURL}/python`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ message: question }),
      });

      if (!response.ok) {
        throw new Error('Network response was not ok');
      }

      const data = await response.json();
      setAnswer(data.payload.message);
      data.payload.chunks.map((chunk) => {
        let url = "https://www.pennnationalinsurance.com/" + chunk.metadata.source.split("penn_")[1].split(".txt")[0].replaceAll("_", "/");
        setUrls((prevUrls) => [...new Set([...prevUrls, url])]);
      })
    } catch (error) {
      console.error('There was a problem with the fetch operation:', error);
      setAnswer('Sorry, something went wrong. Please try again later.');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="App">
      <div className="container">
        <h1>Penn National Insurance Assistant</h1>
        <div className="form-group">
          <label>Your Question</label>
          <textarea 
            className="question-input"
            value={question}
            onChange={(e) => setQuestion(e.target.value)}
            placeholder="Type your question here..."
            rows="4"
            disabled={isLoading}
          />
        </div>
	  {!answer && ( 
        <button 
          className="submit-button"
          onClick={handleSubmit}
          disabled={isLoading || !question.trim()}
        >
          {isLoading ? 'Processing...' : 'Get Answer'}
        </button>
	  )}
        
        <p className="helper-text">
          Ask questions about Penn National Insurance policies and coverage options.
        </p>
        
        {answer && (
          <div className="response-container">
            <label>Response:</label>
            <div className="response-box">
              {answer}
            </div>
            <button 
              className="submit-button"
              onClick={() => {
                setQuestion('');
                setAnswer('');
              }}
            >
              Ask New Question
            </button>
		<p> urls where this info came from </p>
		<ul>
		{urls.map((url) => (
			<li key={url}>
			<a href={url}> {url} </a>
			</li>
		))}
		</ul>
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
