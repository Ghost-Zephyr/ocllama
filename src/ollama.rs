use std::{collections::HashMap, sync::Arc};
use crate::discord::UserId;
use tokio::sync::Mutex;
pub use ollama_rs::{
    generation::completion::{
        request::GenerationRequest, GenerationResponse, GenerationContext,
    },
    models::{
        pull::PullModelStatus,
        LocalModel, ModelInfo
    }, Ollama
};

pub struct Ocllama {
    contexts: Arc<Mutex<HashMap<UserId, UserContext>>>,
    inner: Ollama,
}

#[derive(Clone)]
struct UserContext {
    pub gen_ctx: Option<GenerationContext>,
    pub model: String,
}

pub struct Settings {
    pub model: String,
}

impl From<&UserContext> for Settings {
    fn from(user_ctx: &UserContext) -> Self {
        Self {
            model: user_ctx.model.clone()
        }
    }
}

impl Ocllama {
    pub fn new(uri: String, port: u16) -> Self {
        let inner = Ollama::new(uri, port);
        Self { inner, contexts: Arc::new(Mutex::new(HashMap::new())) }
    }

    async fn context(&self, user_id: &UserId) -> UserContext {
        let contexts = self.contexts.lock().await;
        if let Some(ctx) = contexts.get(user_id) {
            return ctx.clone();
        }
        UserContext::default()
    }

    pub async fn query(&self, user_id: &UserId, query: String) -> Result<GenerationResponse, String> {
        let mut user_ctx = self.context(user_id).await;
        let mut request = GenerationRequest::new(user_ctx.model.clone(), query);
        if let Some(context) = &user_ctx.gen_ctx {
            request = request.context(context.clone());
        }

        let res = self.inner.generate(request).await?;

        if let Some(final_data) = res.final_data.clone() {
            user_ctx.gen_ctx = Some(final_data.context);
            let mut contexts = self.contexts.lock().await;
            contexts.insert(*user_id, user_ctx);
        }
        Ok(res)
    }

    pub async fn setmodel(&self, user_id: &UserId, model: String) -> Result<String, String> {
        let mut models = self.list().await?;
        models.retain(|local_model| local_model.name == model);
        if models.is_empty() {
            return Err("Requested model not found".to_owned());
        }
        let ctx = UserContext {
            model: model.clone(),
            ..UserContext::default()
        };
        let mut contexts = self.contexts.lock().await;
        contexts.insert(*user_id, ctx);
        Ok(model)
    }

    pub async fn add_model(&self, model: String) -> Result<PullModelStatus, String> {
        match self.inner.pull_model(model, false).await {
            Err(err) => Err(format!("{err:?}")),
            Ok(status) => Ok(status),
        }
    }

    pub async fn delete_model(&self, model: String) -> Result<String, String> {
        match self.inner.delete_model(model).await {
            Err(err) => Err(format!("{err:?}")),
            Ok(_) => Ok("deleted".to_owned()),
        }
    }

    pub async fn list(&self) -> Result<Vec<LocalModel>, String> {
        match self.inner.list_local_models().await {
            Err(err) => Err(format!("{err:?}")),
            Ok(models) => Ok(models),
        }
    }

    pub async fn info(&self, model: String) -> Result<ModelInfo, String> {
        match self.inner.show_model_info(model).await {
            Err(err) => Err(format!("{err:?}")),
            Ok(info) => Ok(info),
        }
    }

    pub async fn settings(&self, user_id: &UserId) -> Settings {
        (&self.context(user_id).await).into()
    }
}

impl UserContext {
}

impl Default for UserContext {
    fn default() -> Self {
        Self {
            gen_ctx: None, model: "phi:latest".to_owned()
        }
    }
}
