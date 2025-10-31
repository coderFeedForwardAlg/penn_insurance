import os
import httpx
# from openai import OpenAI
# from dotenv import load_dotenv
from typing import Dict, List, Optional, Any
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
from dotenv import load_dotenv
from langchain_chroma import Chroma
from langchain_openai import OpenAIEmbeddings
import chromadb
import uvicorn
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)



load_dotenv()

class Message(BaseModel):
    role: str
    content: str
    refusal: Optional[str] = None
    annotations: List[Any] = Field(default_factory=list)

class Choice(BaseModel):
    index: int
    message: Message
    logprobs: Optional[Any] = None
    finish_reason: str

class Usage(BaseModel):
    prompt_tokens: int
    completion_tokens: int
    total_tokens: int
    prompt_tokens_details: Dict[str, int]
    completion_tokens_details: Dict[str, int]

class ChatResponse(BaseModel):
    id: str
    object: str
    created: int
    model: str
    choices: List[Choice]
    usage: Usage
    system_fingerprint: Optional[str] = None

class ChatMessage(BaseModel):
    message: str

def query_similar_chunks_with_relevance(query: str, k: int = 3, api_key: str = None):
    """Query the database for k most similar chunks with relevance scores (0-1)."""
    client = chromadb.PersistentClient(path="./chroma_langchain_db")
    print("Client:", client)
    
    # Initialize embeddings with the API key
    embeddings = OpenAIEmbeddings(model="text-embedding-3-large", openai_api_key=api_key)
    
    try:
        vector_store = Chroma(
            collection_name="insurance_collection",
            embedding_function=embeddings,
            client=client,
        )
    except Exception as e:
        print("Error creating vector store:", e)
        raise HTTPException(status_code=500, detail=str(e))
    
    # Get k most similar documents with relevance scores
    similar_docs_with_scores = vector_store.similarity_search_with_relevance_scores(query, k=k)
    
    results = []
    for doc, relevance_score in similar_docs_with_scores:
        results.append({
            'text': doc.page_content,
            'metadata': doc.metadata,
            'relevance_score': relevance_score
        })
    
    return results


app = FastAPI()
@app.get("/health")
async def health_check() -> Dict[str, str]:
    return {"status": "healthy"}

@app.post("/chat")
async def chat(message: ChatMessage) -> Dict[str, Any]:
    logger.info("/chat being called")
    key = os.environ.get("OPENAI_API_KEY")
    if not key:
        logger.info("no api key")
        raise HTTPException(status_code=500, detail="OpenAI API key not configured")
    try:
        logger.info("trying to get chunks and hit open ai")
        similar_chunks = query_similar_chunks_with_relevance(message.message, api_key=key)
        logger.info("similar chunks: %s", similar_chunks)
        # print("Similar chunks:", similar_chunks)
        response = httpx.post(
            "https://api.openai.com/v1/chat/completions",
            headers={
                "Authorization": f"Bearer {key}",
                "Content-Type": "application/json"
            },
            json={
                "model": "gpt-3.5-turbo",
                "messages": [{"role": "user", "content": message.message + "\n\n" + "you are a insurance expert and you are given context from the penn national insurance website. Use this context to answer the question and give exact quotes whenever possible: " + str(similar_chunks)}]
            }
        )
        logger.info("the raw response: %s", response)
        response.raise_for_status()
        logger.info("Response: %s", response.text)
        data = response.json()
        logger.info("Data: %s", data)
        logger.info("\n\n\n")
        return { "message": data['choices'][0]['message']['content'], "chunks": similar_chunks}
    except httpx.HTTPStatusError as e:
        logger.info("error with https")
        raise HTTPException(status_code=e.response.status_code, detail=str(e))
    except Exception as e:
        logger.info("error with https but different ")
        raise HTTPException(status_code=500, detail=str(e))

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8003)
