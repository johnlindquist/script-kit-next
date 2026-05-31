export type HitlChoiceOption = {
  id: string;
  title: string;
  surface?: string;
  tags?: string[];
  description: string;
  goal?: string;
  risk?: string;
  suggestedProofCommands?: string[];
  expectedEvidence?: {
    pass: string;
    fail: string;
  };
};

export type HitlChoiceJob = {
  jobId: string;
  title: string;
  description?: string;
  createdAt?: string;
  options: HitlChoiceOption[];
};

export type HitlChoiceSubmissionInput = {
  jobId: string;
  selectedOptionIds: string[];
  optionFeedback?: Record<string, string>;
  overallFeedback?: string;
  client?: Record<string, unknown>;
};

export type HitlChoiceDraft = HitlChoiceSubmissionInput & {
  clientId: string;
  savedAt: string;
  selectedOptions: HitlChoiceOption[];
};

export type HitlChoiceSubmission = HitlChoiceSubmissionInput & {
  submissionId: string;
  submittedAt: string;
  selectedOptions: HitlChoiceOption[];
};
